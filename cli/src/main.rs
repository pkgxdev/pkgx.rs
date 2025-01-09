mod args;
mod execve;
mod help;

use std::{error::Error, fmt::Write, sync::Arc, time::Duration};

use execve::execve;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use libpkgx::{
    config::Config, env, hydrate::hydrate, install_multi, pantry_db, resolve::resolve, sync,
    types::PackageReq, utils,
};
use rusqlite::Connection;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args::Args {
        plus,
        mut args,
        mode,
        flags,
        find_program,
    } = args::parse();

    match mode {
        args::Mode::Help => {
            println!("{}", help::usage());
            return Ok(());
        }
        args::Mode::Version => {
            println!("pkgx {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        args::Mode::X => (),
    }

    let config = Config::new()?;

    let cache_dir = config.pantry_dir.parent().unwrap();
    std::fs::create_dir_all(cache_dir)?;
    let mut conn = Connection::open(cache_dir.join("pantry.db"))?;

    let spinner = if flags.silent {
        None
    } else {
        let spinner = indicatif::ProgressBar::new_spinner();
        spinner.enable_steady_tick(Duration::from_millis(100));
        Some(spinner)
    };

    let did_sync = if sync::should(&config) {
        if let Some(spinner) = &spinner {
            spinner.set_message("syncing pkg-db…");
        }
        sync::replace(&config, &mut conn).await?;
        true
    } else {
        false
    };

    if let Some(spinner) = &spinner {
        spinner.set_message("resolving pkg graph…");
    }

    let mut pkgs = vec![];

    if find_program {
        let PackageReq {
            constraint,
            project: cmd,
        } = PackageReq::parse(&args[0])?;

        args[0] = cmd.clone(); // invoke eg. `node` rather than eg. `node@20`

        let project = match which(&cmd, &conn).await {
            Err(WhichError::CmdNotFound(cmd)) => {
                if !did_sync {
                    if let Some(spinner) = &spinner {
                        let msg = format!("{} not found, syncing…", cmd);
                        spinner.set_message(msg);
                    }
                    // cmd not found ∴ sync in case it is new
                    sync::replace(&config, &mut conn).await?;
                    if let Some(spinner) = &spinner {
                        spinner.set_message("resolving pkg graph…");
                    }
                    which(&cmd, &conn).await
                } else {
                    Err(WhichError::CmdNotFound(cmd))
                }
            }
            Err(err) => Err(err),
            Ok(project) => Ok(project),
        }?;

        pkgs.push(PackageReq {
            project,
            constraint,
        });
    }

    for pkgspec in plus {
        let PackageReq {
            project: project_or_cmd,
            constraint,
        } = PackageReq::parse(&pkgspec)?;
        if config
            .pantry_dir
            .join("projects")
            .join(project_or_cmd.clone())
            .is_dir()
        {
            pkgs.push(PackageReq {
                project: project_or_cmd,
                constraint,
            });
        } else {
            let project = which(&project_or_cmd, &conn).await?;
            pkgs.push(PackageReq {
                project,
                constraint,
            });
        }
    }

    let companions = pantry_db::companions_for_projects(
        &pkgs
            .iter()
            .map(|project| project.project.clone())
            .collect::<Vec<_>>(),
        &conn,
    )?;

    pkgs.extend(companions);

    let graph = hydrate(&pkgs, |project| {
        pantry_db::deps_for_project(&project, &conn)
    })
    .await?;

    let resolution = resolve(graph, &config).await?;

    let spinner_clone = spinner.clone();
    let clear_progress_bar = move || {
        if let Some(spinner) = spinner_clone {
            spinner.finish_and_clear();
        }
    };

    let mut installations = resolution.installed;
    if !resolution.pending.is_empty() {
        let pb = spinner.map(|spinner| {
            configure_bar(&spinner);
            Arc::new(MultiProgressBar { pb: spinner })
        });
        let installed = install_multi::install_multi(&resolution.pending, &config, pb).await?;
        installations.extend(installed);
    }

    let env = env::map(&installations);

    if !args.is_empty() {
        let pkgx_lvl = std::env::var("PKGX_LVL")
            .unwrap_or("0".to_string())
            .parse()
            .unwrap_or(0)
            + 1;
        if pkgx_lvl >= 10 {
            return Err("PKGX_LVL exceeded: https://github.com/orgs/pkgxdev/discussions/11".into());
        }

        let cmd = if find_program {
            utils::find_program(&args.remove(0), &env["PATH"]).await?
        } else if args[0].contains('/') {
            // user specified a path to program which we should use
            args.remove(0)
        } else {
            // user wants a system tool, eg. pkgx +wget -- git clone
            // NOTE we still check the injected PATH since they may have added the tool anyway
            // it’s just this route allows the user to get a non-error for delegating through to the system
            let mut paths = vec![];
            if let Some(pkgpaths) = env.get("PATH") {
                paths.append(&mut pkgpaths.clone());
            }
            if let Ok(syspaths) = std::env::var("PATH") {
                paths.extend(
                    syspaths
                        .split(':')
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>(),
                );
            }
            utils::find_program(&args.remove(0), &paths).await?
        };
        let env = env::mix(env);
        let mut env = env::mix_runtime(&env, &installations, &conn)?;

        // fork bomb protection
        env.insert("PKGX_LVL".to_string(), pkgx_lvl.to_string());

        clear_progress_bar();

        execve(cmd, args, env)
    } else if !env.is_empty() {
        clear_progress_bar();

        if !flags.json {
            let env = env.iter().map(|(k, v)| (k.clone(), v.join(":"))).collect();
            let env = env::mix_runtime(&env, &installations, &conn)?;
            for (key, value) in env {
                println!("{}=\"{}${{{}:+:${}}}\"", key, value, key, key);
            }
        } else {
            let json = json!({
                "pkgs": installations,
                "env": env
            });
            println!("{}", json);
        }
        Ok(())
    } else {
        clear_progress_bar();
        eprintln!("{}", help::usage());
        std::process::exit(2);
    }
}

#[derive(Debug)]
pub enum WhichError {
    CmdNotFound(String),
    MultipleProjects(String, Vec<String>),
    DbError(rusqlite::Error),
}

impl std::fmt::Display for WhichError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WhichError::CmdNotFound(cmd) => write!(f, "cmd not found: {}", cmd),
            WhichError::MultipleProjects(cmd, projects) => {
                write!(f, "multiple projects found for {}: {:?}", cmd, projects)
            }
            WhichError::DbError(err) => write!(f, "db error: {}", err),
        }
    }
}

impl std::error::Error for WhichError {}

async fn which(cmd: &String, conn: &Connection) -> Result<String, WhichError> {
    let candidates = pantry_db::which(cmd, conn).map_err(WhichError::DbError)?;
    if candidates.len() == 1 {
        Ok(candidates[0].clone())
    } else if candidates.is_empty() {
        return Err(WhichError::CmdNotFound(cmd.clone()));
    } else {
        return Err(WhichError::MultipleProjects(cmd.clone(), candidates));
    }
}

struct MultiProgressBar {
    pb: ProgressBar,
}

impl libpkgx::install_multi::ProgressBarExt for MultiProgressBar {
    fn inc(&self, n: u64) {
        self.pb.inc(n);
    }

    fn inc_length(&self, n: u64) {
        self.pb.inc_length(n);
    }
}

// ProgressBar is Send + Sync
unsafe impl Send for MultiProgressBar {}
unsafe impl Sync for MultiProgressBar {}

fn configure_bar(pb: &ProgressBar) {
    pb.set_length(1);
    pb.set_style(
        ProgressStyle::with_template(
            "{elapsed:.dim} ❲{wide_bar:.red}❳ {percent}% {bytes_per_sec:.dim} {bytes:.dim}",
        )
        .unwrap()
        .with_key("elapsed", |state: &ProgressState, w: &mut dyn Write| {
            let s = state.elapsed().as_secs_f64();
            let precision = precision(s);
            write!(w, "{:.precision$}s", s, precision = precision).unwrap()
        })
        .with_key("bytes", |state: &ProgressState, w: &mut dyn Write| {
            let (right, divisor) = pretty_size(state.len().unwrap());
            let left = state.pos() as f64 / divisor as f64;
            let leftprecision = precision(left);
            write!(
                w,
                "{:.precision$}/{}",
                left,
                right,
                precision = leftprecision
            )
            .unwrap()
        })
        .progress_chars("⚯ "),
    );
    pb.enable_steady_tick(Duration::from_millis(50));
}

fn pretty_size(n: u64) -> (String, u64) {
    let units = ["B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB", "ZiB", "YiB"];
    let mut size = n as f64;
    let mut i = 0;
    let mut divisor = 1;

    while size > 1024.0 && i < units.len() - 1 {
        size /= 1024.0;
        i += 1;
        divisor *= 1024;
    }

    let formatted = format!(
        "{:.precision$} {}",
        size,
        units[i],
        precision = precision(size)
    );

    (formatted, divisor)
}

fn precision(n: f64) -> usize {
    2 - (n.log10().clamp(0.0, 2.0) as usize)
}

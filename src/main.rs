mod args;
mod cellar;
mod config;
mod env;
mod execve;
mod help;
mod hydrate;
mod install;
mod install_multi;
mod inventory;
mod pantry;
mod pantry_db;
mod resolve;
mod sync;
mod types;
mod utils;

use std::error::Error;

use config::Config;
use execve::execve;
use hydrate::hydrate;
use resolve::resolve;
use types::PackageReq;

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
        _ => (),
    }

    let config = Config::new()?;

    let conn = if sync::should(&config) {
        sync::replace(&config).await?
    } else {
        rusqlite::Connection::open(config.pantry_dir.parent().unwrap().join("pantry.db"))?
    };

    let mut pkgs = vec![];
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
            let project = which(&project_or_cmd, &conn)?;
            pkgs.push(PackageReq {
                project,
                constraint,
            });
        }
    }

    if find_program {
        let PackageReq {
            constraint,
            project: cmd,
        } = PackageReq::parse(&args[0])?;
        args[0] = cmd.clone(); // invoke eg. `node` rather than `node@20`
        let project = which(&cmd, &conn)?;
        pkgs.push(PackageReq {
            project,
            constraint,
        });
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
    let mut installations = resolution.installed;
    if !resolution.pending.is_empty() {
        let installed =
            install_multi::install_multi(&resolution.pending, &config, flags.silent).await?;
        installations.extend(installed);
    }

    let env = env::map(&installations);

    if args.is_empty() {
        let env = env.iter().map(|(k, v)| (k.clone(), v.join(":"))).collect();
        let env = env::mix_runtime(&env, &installations, &conn)?;
        for (key, value) in env {
            println!("{}=\"{}${{{}:+:${}}}\"", key, value, key, key);
        }
        Ok(())
    } else {
        let pkgx_lvl = std::env::var("PKGX_LVL")
            .unwrap_or("0".to_string())
            .parse()
            .unwrap_or(0)
            + 1;
        if pkgx_lvl >= 10 {
            return Err("PKGX_LVL exceeded: https://github.com/orgs/pkgxdev/discussions/11".into());
        }

        let cmd = if find_program {
            utils::find_program(&args.remove(0), &env["PATH"], &config).await?
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
            utils::find_program(&args.remove(0), &paths, &config).await?
        };
        let env = env::mix(env);
        let mut env = env::mix_runtime(&env, &installations, &conn)?;
        env.insert("PKGX_LVL".to_string(), pkgx_lvl.to_string());
        execve(cmd, args, env)
    }
}

fn which(cmd: &String, conn: &rusqlite::Connection) -> Result<String, Box<dyn Error>> {
    let candidates = pantry_db::which(cmd, conn)?;
    if candidates.len() == 1 {
        Ok(candidates[0].clone())
    } else if candidates.is_empty() {
        return Err(format!("cmd not found: {}", cmd).into());
    } else {
        return Err(format!(
            "cmd `{}` provided by multiple projects: {:?}",
            cmd, candidates
        )
        .into());
    }
}

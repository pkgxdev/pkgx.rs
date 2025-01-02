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

use config::Config;
use execve::execve;
use hydrate::hydrate;
use resolve::resolve;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args::Args {
        mut plus,
        mut args,
        mode,
        flags,
        find_program,
    } = args::parse();

    match mode {
        args::Mode::Help => {
            println!("{}", help::usage(flags.verbosity));
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
        sync::replace(&config).await?;
        pantry_db::cache(&config)?
    } else {
        rusqlite::Connection::open(config.pantry_dir.parent().unwrap().join("pantry.db"))?
    };

    if find_program {
        plus.push(args[0].clone());
    }

    let pkgs = pantry_db::convert(plus, &conn)?;

    let graph = hydrate(&pkgs, |project| {
        pantry_db::deps_for_project(&project, &conn)
    })
    .await?;

    let resolution = resolve(graph, &config).await?;
    let mut installations = resolution.installed;
    if !resolution.pending.is_empty() {
        let installed = install_multi::install_multi(&resolution.pending, &config).await?;
        installations.extend(installed);
    }

    let env = env::map(installations);

    if args.is_empty() {
        for (key, values) in env {
            println!("{}={}", key, values.join(":"));
        }
        Ok(())
    } else {
        let cmd = if find_program {
            utils::find_program(&args.remove(0), &env["PATH"])?
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
            utils::find_program(&args.remove(0), &paths)?
        };
        let env = env::mix(env);
        execve(cmd, args, env)
    }
}

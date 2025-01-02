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
use utils::find_program;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (mut plus, args, mode, flags) = args::parse();

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
        rusqlite::Connection::open(&config.pantry_dir.parent().unwrap().join("pantry.db"))?
    };

    if !args.is_empty() && !args[0].contains('/') {
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
    } else if let Some(cmd) = find_program(&args[0], &env["PATH"]) {
        let env = env::mix(env);
        execve(cmd, args[1..].to_vec(), env)
    } else {
        Err(format!("cmd not found: {}", args[0]).into())
    }
}

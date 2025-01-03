use std::{error::Error, os::unix::fs::PermissionsExt, path::Path};

use crate::config::Config;

pub async fn find_program(
    arg: &str,
    paths: &Vec<String>,
    config: &Config,
) -> Result<String, Box<dyn Error>> {
    if let Ok(cmd) = find_program_internal(arg, paths) {
        Ok(cmd)
    } else {
        // possibly this program is newer than the last sync
        //TODO this is pretty slow
        crate::sync::replace(config).await?;
        find_program_internal(arg, paths)
    }
}

fn find_program_internal(arg: &str, paths: &Vec<String>) -> Result<String, Box<dyn Error>> {
    if arg.starts_with("/") {
        return Ok(arg.to_string());
    } else if arg.contains("/") {
        return Ok(std::env::current_dir()
            .unwrap()
            .join(arg)
            .to_str()
            .unwrap()
            .to_string());
    }
    for path in paths {
        let full_path = Path::new(&path).join(arg);
        if let Ok(metadata) = full_path.metadata() {
            if full_path.is_file() && (metadata.permissions().mode() & 0o111 != 0) {
                return Ok(full_path.to_str().unwrap().to_string());
            }
        }
    }
    Err(format!("cmd not found: {}", arg).into())
}

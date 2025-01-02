use std::{os::unix::fs::PermissionsExt, path::Path};

pub fn find_program(arg: &str, paths: &Vec<String>) -> Option<String> {
    if arg.starts_with("/") {
        return Some(arg.to_string());
    } else if arg.contains("/") {
        return Some(
            std::env::current_dir()
                .unwrap()
                .join(arg)
                .to_str()
                .unwrap()
                .to_string(),
        );
    }
    for path in paths {
        let full_path = Path::new(&path).join(arg);
        if let Ok(metadata) = full_path.metadata() {
            if full_path.is_file() && (metadata.permissions().mode() & 0o111 != 0) {
                return Some(full_path.to_str().unwrap().to_string());
            }
        }
    }
    None
}

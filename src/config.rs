use std::env;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Config {
    pub pantry_dir: PathBuf,
}

impl Config {
    pub fn new() -> io::Result<Self> {
        let pantry_dir = Self::get_pantry_dir()?;
        Ok(Self { pantry_dir })
    }

    fn get_pantry_dir() -> io::Result<PathBuf> {
        if let Ok(env_dir) = env::var("PKGX_PANTRY_DIR") {
            let path = PathBuf::from(env_dir);
            if !path.is_absolute() {
                return Ok(env::current_dir()?.join(path));
            } else {
                return Ok(path);
            }
        }
        return Ok(dirs_next::data_dir().unwrap().join("pkgx/pantry2"));
    }
}

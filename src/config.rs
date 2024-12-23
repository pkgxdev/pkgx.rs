use std::env;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Config {
    pub pantry_dir: PathBuf,
    pub dist_url: String,
}

impl Config {
    pub fn new() -> io::Result<Self> {
        let pantry_dir = get_pantry_dir()?;
        let dist_url = get_dist_url();
        Ok(Self { pantry_dir, dist_url })
    }
}

fn get_dist_url() -> String {
  if let Ok(env_url) = env::var("PKGX_DIST_URL") {
      return env_url;
  }
  return "https://dist.pkgx.dev".to_string();
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

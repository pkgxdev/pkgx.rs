mod config;

use crate::config::Config;
use std::io;

fn main() -> io::Result<()> {
    let config = Config::new()?;
    println!("Pantry directory: {:?}", config.pantry_dir);
    Ok(())
}

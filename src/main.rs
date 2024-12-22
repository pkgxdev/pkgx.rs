mod config;
mod pantry;
mod sync;

use pantry::PantryEntry;
use crate::config::Config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new()?;
    sync::replace(&config)?;

    let pkg = PantryEntry::new("git-scm.org".to_string(), &config)?;
    println!("pkg {:?}", pkg);

    Ok(())
}

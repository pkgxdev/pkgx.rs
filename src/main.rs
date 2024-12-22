mod inventory;
mod hydrate;
mod resolve;
mod cellar;
mod config;
mod pantry;
mod types;
mod sync;

use pantry::PantryEntry;
use hydrate::hydrate;
use config::Config;

fn main() {
  let rt = tokio::runtime::Runtime::new().unwrap();
  rt.block_on(async_main()).unwrap();
}

async fn async_main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new()?;
    sync::replace(&config).await?;

    let pkg = PantryEntry::new("git-scm.org".to_string(), &config)?;
    let graph = hydrate(pkg.dependencies, |pkgname| Ok(PantryEntry::new(pkgname, &config)?.dependencies)).await;
    println!("pkg {:?}", graph);

    Ok(())
}

use std::collections::HashMap;
use crate::config::Config;
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct PantryEntry {
    dependencies: HashMap<String, String>,
}

impl PantryEntry {
  pub fn new(pkgname: String, config: &Config) -> Result<Self, Box<dyn std::error::Error>> {
    let path = config.pantry_dir.join(format!("projects/{}/package.yml", pkgname));
    let content = fs::read_to_string(path)?;
    let entry: PantryEntry = serde_yaml::from_str(&content)?;
    Ok(entry)
  }
}

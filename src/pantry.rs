use std::collections::HashMap;
use crate::{config::Config, types::PackageReq};
use serde::Deserialize;
use semver::VersionReq;
use std::fs;

pub struct PantryEntry {
  pub dependencies: Vec<PackageReq>,
}

impl PantryEntry {
  pub fn new(pkgname: String, config: &Config) -> Result<Self, Box<dyn std::error::Error>> {
    let entry = RawPantryEntry::new(pkgname, config)?;
    let dependencies = entry.dependencies.iter()
      .map(|(k, v)| PackageReq{ project: k.clone(), constraint: VersionReq::parse(v).unwrap() })
      .collect();
    Ok(Self { dependencies })
  }
}

#[derive(Debug, Deserialize)]
struct RawPantryEntry {
    dependencies: HashMap<String, String>,
}

impl RawPantryEntry {
  fn new(pkgname: String, config: &Config) -> Result<Self, Box<dyn std::error::Error>> {
    let path = config.pantry_dir.join(format!("projects/{}/package.yml", pkgname));
    let content = fs::read_to_string(path)?;
    let entry: RawPantryEntry = serde_yaml::from_str(&content)?;
    Ok(entry)
  }
}

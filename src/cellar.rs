use crate::types::{Installation, Package, PackageReq};
use crate::config::Config;
use std::cmp::Ordering;
use std::path::PathBuf;
use std::error::Error;
use semver::Version;
use tokio::fs;

pub async fn ls(project: &str, config: &Config) -> Result<Vec<Installation>, Box<dyn std::error::Error>> {
  let d = config.pantry_dir.join("projects").join(project);

  if !fs::metadata(&d).await?.is_dir() {
      return Ok(vec![]);
  }

  let mut rv = vec![];
  let mut entries = fs::read_dir(&d).await?;
  while let Some(entry) = entries.next_entry().await? {
      let path = entry.path();
      let name = entry.file_name().to_string_lossy().to_string();

      if !fs::metadata(&path).await?.is_dir() {
          continue;
      }
      if !name.starts_with('v') || name == "var" {
          continue;
      }

      match Version::parse(&name[1..]) {
          Ok(version) => {
              if vacant(&path).await? {
                  continue;
              }
              rv.push(Installation {
                  path,
                  pkg: Package {
                      project: project.to_string(),
                      version,
                  },
              });
          }
          Err(_) => {
              // Ignore invalid version directories
          }
      }
  }

  rv.sort_by(|a, b| compare_packages(&a.pkg, &b.pkg));
  Ok(rv)
}

/// possibly archaic check for failed installations
async fn vacant(path: &PathBuf) -> Result<bool, Box<dyn std::error::Error>> {
  // Check if the path exists and is a directory
  let metadata = match fs::metadata(path).await {
      Ok(meta) => meta,
      Err(_) => return Ok(true), // Treat non-existent paths as vacant
  };

  if !metadata.is_dir() {
      return Ok(true);
  }

  // Iterate over directory contents
  let mut entries = fs::read_dir(path).await?;
  while let Some(_) = entries.next_entry().await? {
      return Ok(false); // Found at least one file/directory
  }

  Ok(true) // Directory is empty
}

// Utility to compare packages
fn compare_packages(a: &Package, b: &Package) -> Ordering {
  match a.project.cmp(&b.project) {
      Ordering::Equal => a.version.cmp(&b.version),
      other => other,
  }
}

#[derive(Debug)]
pub struct InstallationNotFoundError(pub PackageReq);

impl std::fmt::Display for InstallationNotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Installation not found for {:?}", self.0)
    }
}

impl Error for InstallationNotFoundError {}

pub async fn resolve(pkg: PackageReq, config: &Config) -> Result<Installation, Box<dyn Error>> {
    // Get all installations for the project
    let installations = ls(&pkg.project, config).await?;
    let versions: Vec<Version> = installations
        .iter()
        .map(|installation| installation.pkg.version.clone())
        .collect();

    // Find the maximum version that matches the constraint
    if let Some(version) = versions.iter().filter(|v| pkg.constraint.matches(v)).max() {
        // Find the installation that matches the resolved version
        if let Some(installation) = installations
            .into_iter()
            .find(|i| i.pkg.version == *version)
        {
            return Ok(installation);
        }
    }

    // If no matching version is found, return an error
    Err(Box::new(InstallationNotFoundError(pkg)))
}

pub async fn has(pkg: PackageReq, config: &Config) -> Option<Installation> {
  match resolve(pkg, config).await {
      Ok( foo) => { Some(foo) },
      Err(_) => { None } //FIXME only swallow errors for the correct error types
  }
}

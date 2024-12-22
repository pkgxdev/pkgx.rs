use crate::types::{Installation, Package, PackageReq};
use crate::{cellar, inventory};
use crate::config::Config;
use std::error::Error;

#[derive(Debug, Default)]
pub struct Resolution {
  /// fully resolved list (includes both installed and pending)
  pkgs: Vec<Package>,

  /// already installed packages
  installed: Vec<Installation>,

  /// these are the pkgs that aren’t yet installed
  pending: Vec<Package>,
}

//TODO no need to take array since it doesn’t consider anything
pub async fn resolve(reqs: Vec<PackageReq>, config: &Config) -> Result<Resolution, Box<dyn Error>> {
  let mut rv = Resolution::default();

  for req in reqs {
      if let Some(installation) = cellar::has(req.clone(), config).await {
          // If already installed and satisfies the constraint, use it
          rv.installed.push(installation.clone());
          rv.pkgs.push(installation.pkg.clone());
      } else if let Ok(Some(version)) = inventory::select(req.clone(), config).await {
        let pkg = Package {
            project: req.project.clone(),
            version,
        };
        rv.pkgs.push(pkg.clone());
        rv.pending.push(pkg);
      } else {
          return Err(Box::new(ResolveError{pkg: req}));
      }
  }

  Ok(rv)
}

use std::fmt;

#[derive(Debug)]
pub struct ResolveError {
    pub pkg: PackageReq, // Holds the package or requirement
}
impl Error for ResolveError {}

impl fmt::Display for ResolveError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      write!(f, "not-found: pkg: {:?}", self.pkg)
  }
}

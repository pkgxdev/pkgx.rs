use semver::{Version, VersionReq};
use std::fmt;

#[derive(Debug, Clone)]
pub struct Package {
    pub project: String,
    pub version: Version,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageReq {
    pub project: String,
    pub constraint: VersionReq,
}
#[derive(Debug, Clone)]
pub struct Installation {
  pub path: std::path::PathBuf,
  pub pkg: Package,
}

pub enum Host {
  Darwin,
  Linux
}

pub enum Arch {
Arm64,
X86_64
}

pub fn host() -> (Host, Arch) {
  let host = match std::env::consts::OS {
    "macos" => Host::Darwin,
    "linux" => Host::Linux,
    _ => panic!("Unsupported platform")
  };
  let arch = match std::env::consts::ARCH {
    "aarch64" => Arch::Arm64,
    "x86_64" => Arch::X86_64,
    _ => panic!("Unsupported architecture")
  };
  return (host, arch)
}

impl fmt::Display for Host {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      let os_str = match self {
          Host::Linux => "linux",
          Host::Darwin => "darwin",
      };
      write!(f, "{}", os_str)
  }
}

impl fmt::Display for Arch {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      let os_str = match self {
          Arch::Arm64 => "aarch64",
          Arch::X86_64 => "x86-64",
      };
      write!(f, "{}", os_str)
  }
}

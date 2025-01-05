use lazy_static::lazy_static;
use libsemverator::range::Range as VersionReq;
use libsemverator::semver::Semver as Version;
use std::error::Error;
use std::fmt;

lazy_static! {
    static ref PACKAGE_REGEX: Regex = Regex::new(r"^(.+?)([\^=~<>@].+)?$").unwrap();
}

#[derive(Debug, Clone)]
pub struct Package {
    pub project: String,
    pub version: Version,
}

impl fmt::Display for Package {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}={}", self.project, &self.version)
    }
}

#[derive(Debug, Clone)]
pub struct PackageReq {
    pub project: String,
    pub constraint: VersionReq,
}

use regex::Regex;

impl PackageReq {
    pub fn parse(pkgspec: &str) -> Result<Self, Box<dyn Error>> {
        let input = pkgspec.trim();
        let captures = PACKAGE_REGEX
            .captures(input)
            .ok_or_else(|| format!("invalid pkgspec: {}", input))?;

        let project = captures.get(1).unwrap().as_str().to_string();
        let str = if let Some(cap) = captures.get(2) {
            cap.as_str()
        } else {
            "*"
        };
        let constraint = VersionReq::parse(str)?;

        Ok(Self {
            project,
            constraint,
        })
    }
}

impl fmt::Display for PackageReq {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.project, &self.constraint)
    }
}

#[derive(Debug, Clone)]
pub struct Installation {
    pub path: std::path::PathBuf,
    pub pkg: Package,
}

// These are only used per build at present
#[allow(dead_code)]
pub enum Host {
    Darwin,
    Linux,
}

// These are only used per build at present
#[allow(dead_code)]
pub enum Arch {
    Arm64,
    X86_64,
}

pub fn host() -> (Host, Arch) {
    #[cfg(target_os = "macos")]
    let host = Host::Darwin;
    #[cfg(target_os = "linux")]
    let host = Host::Linux;
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    panic!("Unsupported platform");

    #[cfg(target_arch = "aarch64")]
    let arch = Arch::Arm64;
    #[cfg(target_arch = "x86_64")]
    let arch = Arch::X86_64;
    #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
    panic!("Unsupported architecture");

    (host, arch)
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

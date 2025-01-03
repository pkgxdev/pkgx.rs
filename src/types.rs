use lazy_static::lazy_static;
use libsemverator::range::{Constraint, Range as VersionReq};
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
        write!(
            f,
            "{}={}",
            self.project,
            semverator_version_to_string(&self.version)
        )
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

pub fn semverator_version_to_string(version: &Version) -> String {
    version
        .components
        .iter()
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join(".")
}

fn semverator_range_to_string(range: &VersionReq) -> String {
    range
        .set
        .iter()
        .map(|v| match v {
            Constraint::Any => "*".to_string(),
            Constraint::Single(v) => format!("={}", semverator_version_to_string(v)),
            Constraint::Contiguous(v1, v2) => {
                if v2.major == v1.major + 1 && v2.minor == 0 && v2.patch == 0 {
                    let v = chomp(v1);
                    if v1.major == 0 {
                        if v1.components.len() == 1 {
                            "^0".to_string()
                        } else {
                            format!(">={}<1", v)
                        }
                    } else {
                        format!("^{}", v)
                    }
                } else if v2.major == v1.major && v2.minor == v1.minor + 1 && v2.patch == 0 {
                    let v = chomp(v1);
                    format!("~{}", v)
                } else if v2.major == usize::MAX {
                    let v = chomp(v1);
                    format!(">={}", v)
                } else if at(v1, v2) {
                    format!("@{}", semverator_version_to_string(v1))
                } else {
                    format!(">={}<{}", chomp(v1), chomp(v2))
                }
            }
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn at(v1: &Version, v2: &Version) -> bool {
    let mut cc1 = v1.components.clone();
    let cc2 = &v2.components;

    if cc1.len() > cc2.len() {
        return false;
    }

    // Ensure cc1 and cc2 have the same length by appending 0s to cc1
    while cc1.len() < cc2.len() {
        cc1.push(0);
    }

    if last(&cc1) != last(cc2) - 1 {
        return false;
    }

    for i in 0..cc1.len() - 1 {
        if cc1[i] != cc2[i] {
            return false;
        }
    }

    true
}

fn last(arr: &[usize]) -> usize {
    *arr.last().unwrap()
}

fn chomp(v: &Version) -> String {
    let result = v.raw.trim_end_matches(".0");
    if result.is_empty() {
        "0".to_string()
    } else {
        result.to_string()
    }
}

impl fmt::Display for PackageReq {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}",
            self.project,
            semverator_range_to_string(&self.constraint)
        )
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

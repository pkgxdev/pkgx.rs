use crate::{config::Config, types::PackageReq};
use libsemverator::range::Range as VersionReq;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub struct PantryEntry {
    pub project: String,
    pub deps: Vec<PackageReq>,
    pub programs: Vec<String>,
    pub companions: Vec<String>,
    pub env: HashMap<String, String>,
}

impl PantryEntry {
    fn from_path(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let project = path
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        Self::from_raw_entry(RawPantryEntry::from_path(path)?, project)
    }

    fn from_raw_entry(
        entry: RawPantryEntry,
        project: String,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        if let Some(deps) = entry.dependencies {
            let deps = deps
                .iter()
                .map(|(k, v)| {
                    // if v is a number, prefix with ^
                    let v = if v.chars().next().unwrap().is_ascii_digit() {
                        format!("^{}", v)
                    } else {
                        v.clone()
                    };
                    VersionReq::parse(&v).map(|constraint| PackageReq {
                        project: k.clone(),
                        constraint,
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;

            let env = HashMap::new();

            Ok(Self {
                deps,
                project,
                env,
                companions: vec![],
                programs: vec![],
            })
        } else {
            Ok(Self {
                deps: vec![],
                project,
                env: HashMap::new(),
                companions: vec![],
                programs: vec![],
            })
        }
    }
}

pub struct PackageEntryIterator {
    stack: Vec<PathBuf>, // stack for directories to visit
}

impl PackageEntryIterator {
    pub fn new(pantry_dir: PathBuf) -> Self {
        Self {
            stack: vec![pantry_dir.clone()],
        }
    }
}

impl Iterator for PackageEntryIterator {
    type Item = PantryEntry;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(path) = self.stack.pop() {
            if path.is_dir() {
                // push subdirectories and files into the stack
                if let Ok(entries) = fs::read_dir(&path) {
                    for entry in entries.flatten() {
                        self.stack.push(entry.path());
                    }
                }
            } else if path.file_name() == Some("package.yml".as_ref()) {
                if let Ok(entry) = PantryEntry::from_path(&path) {
                    return Some(entry);
                }
            }
        }
        None
    }
}

pub fn ls(config: &Config) -> PackageEntryIterator {
    PackageEntryIterator::new(config.pantry_dir.clone())
}

#[derive(Debug, Deserialize)]
struct RawPantryEntry {
    dependencies: Option<HashMap<String, String>>,
}

impl RawPantryEntry {
    fn from_path(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        Ok(serde_yaml::from_str(&content)?)
    }
}

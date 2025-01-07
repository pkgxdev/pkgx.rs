use crate::{config::Config, types::PackageReq};
use libsemverator::range::Range as VersionReq;
use serde::Deserialize;
use serde::Deserializer;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub struct PantryEntry {
    pub project: String,
    pub deps: Vec<PackageReq>,
    pub programs: Vec<String>,
    pub companions: Vec<PackageReq>,
    pub env: HashMap<String, String>,
}

impl PantryEntry {
    fn from_path(path: &PathBuf, pantry_dir: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let project = path
            .parent()
            .unwrap()
            .strip_prefix(pantry_dir)
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
        let deps = if let Some(deps) = entry.dependencies {
            deps.0
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
                .collect::<Result<Vec<_>, _>>()?
        } else {
            vec![]
        };

        let programs = if let Some(provides) = entry.provides {
            provides.0
        } else {
            vec![]
        };

        let companions = if let Some(companions) = entry.companions {
            companions
                .0
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
                .collect::<Result<Vec<_>, _>>()?
        } else {
            vec![]
        };

        let env = if let Some(runtime) = entry.runtime {
            runtime.env
        } else {
            HashMap::new()
        };

        Ok(Self {
            deps,
            project,
            env,
            companions,
            programs,
        })
    }
}

pub struct PackageEntryIterator {
    stack: Vec<PathBuf>, // stack for directories to visit
    pantry_dir: PathBuf,
}

impl PackageEntryIterator {
    pub fn new(pantry_dir: PathBuf) -> Self {
        Self {
            stack: vec![pantry_dir.clone()],
            pantry_dir,
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
                if let Ok(entry) = PantryEntry::from_path(&path, &self.pantry_dir) {
                    return Some(entry);
                } else if cfg!(debug_assertions) {
                    eprintln!("parse failure: {:?}", path);
                }
            }
        }
        None
    }
}

pub fn ls(config: &Config) -> PackageEntryIterator {
    PackageEntryIterator::new(config.pantry_dir.join("projects"))
}

#[derive(Debug, Deserialize)]
struct RawPantryEntry {
    dependencies: Option<Deps>,
    provides: Option<Provides>,
    companions: Option<Deps>,
    runtime: Option<Runtime>,
}

#[derive(Debug, Deserialize)]
struct Runtime {
    env: HashMap<String, String>,
}

#[derive(Debug)]
struct Deps(HashMap<String, String>);

impl<'de> Deserialize<'de> for Deps {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize the map as a generic HashMap
        let full_map: HashMap<String, serde_yaml::Value> = Deserialize::deserialize(deserializer)?;

        // Determine the current platform
        #[cfg(target_os = "macos")]
        let platform_key = "darwin";

        #[cfg(target_os = "linux")]
        let platform_key = "linux";

        #[cfg(target_os = "windows")]
        let platform_key = "windows";

        // Create the result map
        let mut result = HashMap::new();

        for (key, value) in full_map {
            if key == "linux" || key == "darwin" || key == "windows" {
                // If the key is platform-specific, only include values for the current platform
                if key == platform_key {
                    if let serde_yaml::Value::Mapping(platform_values) = value {
                        for (k, v) in platform_values {
                            if let (serde_yaml::Value::String(k), serde_yaml::Value::String(v)) =
                                (k, v)
                            {
                                result.insert(k, v);
                            }
                        }
                    }
                }
            } else {
                // Include non-platform-specific keys
                if let serde_yaml::Value::String(v) = value {
                    result.insert(key, v);
                }
            }
        }

        Ok(Deps(result))
    }
}

#[derive(Debug)]
struct Provides(Vec<String>);

impl<'de> Deserialize<'de> for Provides {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Define an enum to capture the possible YAML structures
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum ProvidesHelper {
            List(Vec<String>),
            Map(HashMap<String, Vec<String>>),
        }

        match ProvidesHelper::deserialize(deserializer)? {
            ProvidesHelper::List(list) => Ok(Provides(list)),
            ProvidesHelper::Map(map) => {
                #[cfg(target_os = "macos")]
                let key = "darwin";

                #[cfg(target_os = "linux")]
                let key = "linux";

                if let Some(values) = map.get(key) {
                    Ok(Provides(values.clone()))
                } else {
                    Ok(Provides(Vec::new())) // Return an empty Vec if the key isn't found
                }
            }
        }
    }
}

impl RawPantryEntry {
    fn from_path(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        Ok(serde_yaml::from_str(&content)?)
    }
}

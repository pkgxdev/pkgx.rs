use std::collections::{HashMap, HashSet};

use crate::types::Installation;

pub fn map(installations: Vec<Installation>) -> HashMap<String, Vec<String>> {
    let mut vars: HashMap<&str, OrderedSet<String>> = HashMap::new();

    let projects: HashSet<&str> = installations
        .iter()
        .map(|i| i.pkg.project.as_str())
        .collect();

    let has_cmake = projects.contains("cmake.org");
    let archaic = true;

    let mut rv: HashMap<String, Vec<String>> = HashMap::new();

    for installation in installations {
        for key in &[
            EnvKey::Path,
            EnvKey::Manpath,
            EnvKey::PkgConfigPath,
            EnvKey::LibraryPath,
            EnvKey::LdLibraryPath,
            EnvKey::Cpath,
            EnvKey::XdgDataDirs,
            EnvKey::CmakePrefixPath,
            EnvKey::DyldFallbackLibraryPath,
            EnvKey::SslCertFile,
            EnvKey::Ldflags,
            EnvKey::PkgxDir,
            EnvKey::AclocalPath,
        ] {
            if let Some(suffixes) = suffixes(key) {
                for suffix in suffixes {
                    let path = installation.path.join(suffix).to_string_lossy().to_string();
                    if !path.is_empty() {
                        vars.entry(key.as_ref())
                            .or_insert_with(OrderedSet::new)
                            .add(path);
                    }
                }
            }
        }

        if archaic {
            let lib_path = installation.path.join("lib").to_string_lossy().to_string();
            vars.entry(EnvKey::LibraryPath.as_ref())
                .or_insert_with(OrderedSet::new)
                .add(lib_path);

            let include_path = installation
                .path
                .join("include")
                .to_string_lossy()
                .to_string();
            vars.entry(EnvKey::Cpath.as_ref())
                .or_insert_with(OrderedSet::new)
                .add(include_path);
        }

        if has_cmake {
            vars.entry(EnvKey::CmakePrefixPath.as_ref())
                .or_insert_with(OrderedSet::new)
                .add(installation.path.to_string_lossy().to_string());
        }
    }

    if let Some(library_path) = vars.get(EnvKey::LibraryPath.as_ref()) {
        let paths = library_path.to_vec();
        vars.entry(EnvKey::LdLibraryPath.as_ref())
            .or_insert_with(OrderedSet::new)
            .items
            .extend(paths.clone());

        // We only need to set DYLD_FALLBACK_LIBRARY_PATH on macOS
        #[cfg(target_os = "macos")]
        vars.entry(EnvKey::DyldFallbackLibraryPath.as_ref())
            .or_insert_with(OrderedSet::new)
            .items
            .extend(paths);
    }

    for (key, set) in &vars {
        if !set.is_empty() {
            rv.insert(key.to_string(), set.to_vec());
        }
    }

    rv
}

use strum_macros::{AsRefStr, EnumString};

#[derive(Debug, EnumString, AsRefStr, PartialEq, Eq, Hash, Clone)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
enum EnvKey {
    Path,
    Manpath,
    PkgConfigPath,
    LibraryPath,
    LdLibraryPath,
    Cpath,
    XdgDataDirs,
    CmakePrefixPath,
    DyldFallbackLibraryPath,
    SslCertFile,
    Ldflags,
    PkgxDir,
    AclocalPath,
}

pub struct OrderedSet<T: Eq + std::hash::Hash + Clone> {
    items: Vec<T>,
    set: HashSet<T>,
}

impl<T: Eq + std::hash::Hash + Clone> OrderedSet<T> {
    pub fn new() -> Self {
        OrderedSet {
            items: Vec::new(),
            set: HashSet::new(),
        }
    }

    pub fn add(&mut self, item: T) {
        if self.set.insert(item.clone()) {
            self.items.push(item);
        }
    }

    pub fn to_vec(&self) -> Vec<T> {
        self.items.clone()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}
fn suffixes(key: &EnvKey) -> Option<Vec<&'static str>> {
    match key {
        EnvKey::Path => Some(vec!["bin", "sbin"]),
        EnvKey::Manpath => Some(vec!["man", "share/man"]),
        EnvKey::PkgConfigPath => Some(vec!["share/pkgconfig", "lib/pkgconfig"]),
        EnvKey::XdgDataDirs => Some(vec!["share"]),
        EnvKey::LibraryPath
        | EnvKey::LdLibraryPath
        | EnvKey::DyldFallbackLibraryPath
        | EnvKey::Cpath
        | EnvKey::CmakePrefixPath
        | EnvKey::SslCertFile
        | EnvKey::Ldflags
        | EnvKey::PkgxDir
        | EnvKey::AclocalPath => None,
    }
}

pub fn mix(input: HashMap<String, Vec<String>>) -> HashMap<String, String> {
    let mut rv = HashMap::new();

    //TODO handle empty values etc.

    for (key, mut value) in std::env::vars() {
        if let Some(injected_values) = input.get(&key) {
            value = format!("{}:{}", injected_values.join(":"), value);
        }
        rv.insert(key, value);
    }

    rv
}

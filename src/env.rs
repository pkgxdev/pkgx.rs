use std::collections::{HashMap, HashSet};

use crate::types::Installation;

pub fn map(installations: Vec<Installation>) -> HashMap<String, Vec<String>> {
    let mut vars: HashMap<String, OrderedSet<String>> = HashMap::new();
    let is_mac = cfg!(target_os = "macos");

    let projects: HashSet<_> = installations.iter().map(|i| &i.pkg.project).collect();

    let has_cmake = projects.contains(&"cmake.org".to_string());
    let archaic = true;

    let mut rv: HashMap<String, Vec<String>> = HashMap::new();
    let mut seen = HashSet::new();

    for installation in installations {
        if !seen.insert(installation.pkg.project.clone()) {
            eprintln!("pkgx: env is being duped: {}", installation.pkg.project);
        }

        for key in &[
            EnvKey::PATH,
            EnvKey::MANPATH,
            EnvKey::PkgConfigPath,
            EnvKey::LibraryPath,
            EnvKey::LdLibraryPath,
            EnvKey::CPATH,
            EnvKey::XdgDataDirs,
            EnvKey::CmakePrefixPath,
            EnvKey::DyldFallbackLibraryPath,
            EnvKey::SslCertFile,
            EnvKey::LDFLAGS,
            EnvKey::PkgxDir,
            EnvKey::AclocalPath,
        ] {
            if let Some(suffixes) = suffixes(key) {
                for suffix in suffixes {
                    let path = installation
                        .path
                        .join(&suffix)
                        .to_string_lossy()
                        .to_string();
                    if !path.is_empty() {
                        vars.entry(key.as_ref().to_string())
                            .or_insert_with(OrderedSet::new)
                            .add(path);
                    }
                }
            }
        }

        if archaic {
            let lib_path = installation.path.join("lib").to_string_lossy().to_string();
            vars.entry(EnvKey::LibraryPath.as_ref().to_string())
                .or_insert_with(OrderedSet::new)
                .add(lib_path);

            let include_path = installation
                .path
                .join("include")
                .to_string_lossy()
                .to_string();
            vars.entry(EnvKey::CPATH.as_ref().to_string())
                .or_insert_with(OrderedSet::new)
                .add(include_path);
        }

        if has_cmake {
            vars.entry(EnvKey::CmakePrefixPath.as_ref().to_string())
                .or_insert_with(OrderedSet::new)
                .add(installation.path.to_string_lossy().to_string());
        }
    }

    if let Some(library_path) = vars.get(&EnvKey::LibraryPath.as_ref().to_string()) {
        let paths = library_path.to_vec();
        vars.entry(EnvKey::LdLibraryPath.as_ref().to_string())
            .or_insert_with(OrderedSet::new)
            .items
            .extend(paths.clone());

        if is_mac {
            vars.entry(EnvKey::DyldFallbackLibraryPath.as_ref().to_string())
                .or_insert_with(OrderedSet::new)
                .items
                .extend(paths);
        }
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
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")] // Optional: enforce case style
enum EnvKey {
    PATH,
    MANPATH,
    PkgConfigPath,
    LibraryPath,
    LdLibraryPath,
    CPATH,
    XdgDataDirs,
    CmakePrefixPath,
    DyldFallbackLibraryPath,
    SslCertFile,
    LDFLAGS,
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
        EnvKey::PATH => Some(vec!["bin", "sbin"]),
        EnvKey::MANPATH => Some(vec!["man", "share/man"]),
        EnvKey::PkgConfigPath => Some(vec!["share/pkgconfig", "lib/pkgconfig"]),
        EnvKey::XdgDataDirs => Some(vec!["share"]),
        EnvKey::LibraryPath
        | EnvKey::LdLibraryPath
        | EnvKey::DyldFallbackLibraryPath
        | EnvKey::CPATH
        | EnvKey::CmakePrefixPath
        | EnvKey::SslCertFile
        | EnvKey::LDFLAGS
        | EnvKey::PkgxDir
        | EnvKey::AclocalPath => None,
    }
}

pub fn mix(input: HashMap<String, Vec<String>>) -> HashMap<String, String> {
    let mut rv = HashMap::new();

    for (key, mut values) in input {
        if let Ok(parent_value) = std::env::var(&key) {
            values.push(parent_value.clone());
        }
        rv.insert(key, values.join(":"));
    }

    return rv;
}

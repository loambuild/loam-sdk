use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    process::Command,
};

use cargo_metadata::{camino::Utf8PathBuf, Package, PackageId};
use topological_sort::TopologicalSort;

pub trait PackageExt {
    fn is_dep(&self, key: &DepKind) -> bool;
}

impl PackageExt for Package {
    /// Check if the package has the specified key in its metadata
    fn is_dep(&self, key: &DepKind) -> bool {
        #[allow(clippy::redundant_closure_for_method_calls)]
        self.metadata
            .as_object()
            .and_then(|metadata| metadata.get("loam"))
            .and_then(|riff| riff.as_object())
            .and_then(|riff_object| riff_object.get(&key.to_string()))
            .and_then(|export| export.as_bool())
            .unwrap_or_default()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to find root package with manifest_path {0:?}")]
    RootNotFound(PathBuf),
    #[error("Failed to cargo tree at manifest_path {0:?}")]
    CargoTree(PathBuf),
    #[error("Failed to get parent of {0}")]
    ParentNotFound(PathBuf),
    #[error(transparent)]
    Metadata(#[from] cargo_metadata::Error),
}

pub fn get_deps(manifest_path: &Path) -> Result<Vec<Package>, Error> {
    let metadata = cargo_metadata::MetadataCommand::new()
        .manifest_path(manifest_path)
        .exec()?;

    let p = metadata
        .root_package()
        .ok_or_else(|| Error::RootNotFound(manifest_path.to_path_buf()))?;

    let packages = metadata
        .packages
        .iter()
        .map(|p| (format!("{}v{}", p.name, p.version), p))
        .collect::<HashMap<String, &Package>>();

    let parent = manifest_path
        .parent()
        .ok_or_else(|| Error::ParentNotFound(manifest_path.to_path_buf()))?;
    let output = Command::new("cargo")
        .current_dir(parent)
        .args(["tree", "--prefix", "none", "--edges", "normal"])
        .output()
        .map_err(|_| Error::CargoTree(parent.to_path_buf()))?;
    let stdout = output.stdout;
    let stdout_str = String::from_utf8(stdout).unwrap();

    let mut res = stdout_str
        .lines()
        .filter_map(|line| {
            let s: Vec<&str> = line.split(' ').collect();
            let package_id = format!("{}{}", s[0], s[1]);
            let res = packages.get(&package_id).copied();
            if let Some(r) = &res {
                if r == &p {
                    return None;
                }
            }
            res.map(Clone::clone)
        })
        .collect::<Vec<_>>();
    res.push(p.clone());
    Ok(res)
}

pub fn out_dir(target_dir: &Path, name: &str) -> PathBuf {
    target_dir.join("loam").join(name.replace('-', "_"))
}

pub enum DepKind {
    Riff,
    Contract,
}

impl Display for DepKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DepKind::Riff => write!(f, "riff"),
            DepKind::Contract => write!(f, "contract"),
        }
    }
}

pub fn get_loam_deps(
    manifest_path: &Path,
    kind: DepKind,
) -> Result<Vec<(Utf8PathBuf, PathBuf)>, Error> {
    get_deps(manifest_path)?
        .into_iter()
        .filter(|p| p.is_dep(&kind) || p.manifest_path == manifest_path)
        .map(|p| {
            let version = &p.version;
            let name = &p.name;
            let dir = PathBuf::from(format!("{name}{version}"));
            let out_dir = out_dir(&dir, name);
            let res = (
                p.manifest_path
                    .parent()
                    .ok_or_else(|| Error::ParentNotFound(p.manifest_path.to_path_buf().into()))?
                    .join("src")
                    .join("lib.rs"),
                out_dir,
            );
            Ok(res)
        })
        .collect::<Result<HashSet<_>, Error>>()
        .map(IntoIterator::into_iter)
        .map(Iterator::collect::<Vec<_>>)
}

pub fn get_riff_deps(manifest_path: &Path) -> Result<Vec<(Utf8PathBuf, PathBuf)>, Error> {
    get_loam_deps(manifest_path, DepKind::Riff)
}

pub fn get_contract_deps(manifest_path: &Path) -> Result<Vec<Package>, Error> {
    Ok(get_deps(manifest_path)?
        .into_iter()
        .filter(|p| p.is_dep(&DepKind::Contract) && p.manifest_path != manifest_path)
        .collect())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HashablePackage(Package);

impl Hash for HashablePackage {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.manifest_path.hash(state);
    }
}

pub fn get_workspace(packages: &[Package]) -> Result<Vec<Package>, Error> {
    let mut graph: TopologicalSort<PackageId> = TopologicalSort::new();
    for p in packages {
        let contract_deps = get_contract_deps(&p.manifest_path.clone().into_std_path_buf())?;
        for dep in contract_deps {
            graph.add_dependency(dep.id.clone(), p.id.clone());
        }
        graph.insert(p.id.clone());
    }
    let mut res = Vec::new();
    while let Some(p) = graph.pop() {
        if let Some(contract) = packages.iter().find(|p2| p2.id == p) {
            res.push(contract.clone());
        }
    }
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_loam_deps() {
        let pwd = std::env::current_dir().unwrap();
        println!("{pwd:?}");
        let manifest_path = pwd.join("../../test/normal/Cargo.toml");
        let mut c = cargo_metadata::MetadataCommand::new();
        c.manifest_path(&manifest_path);
        let metadata = c.exec().unwrap();
        let normal = metadata.root_package().unwrap();
        println!("{normal:#?}{}", normal.name);
        let deps = get_deps(&manifest_path).unwrap();
        println!("{deps:#?}\n{}", deps.len());
        let deps = get_riff_deps(&manifest_path).unwrap();
        println!("{deps:#?}\n{}", deps.len());
    }
}

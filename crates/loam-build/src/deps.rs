use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    path::{Path, PathBuf},
    process::Command,
};

use cargo_metadata::{camino::Utf8PathBuf, Package, PackageId};
use topological_sort::TopologicalSort;

/// Retrieves the target directory for a Cargo project and appends "loam" to it.
///
/// This function uses `cargo_metadata` to get the target directory of a Cargo project
/// specified by the given manifest path. It then appends "loam" to this path.
///
/// # Arguments
///
/// * `manifest_path` - A reference to a `Path` representing the location of the Cargo.toml file.
///
/// # Returns
///
/// Returns a `Result` containing:
/// - `Ok(PathBuf)`: The path to the target directory with "loam" appended.
/// - `Err(cargo_metadata::Error)`: If there's an error retrieving the metadata.
///
/// # Errors
///
/// This function will return an error if:
/// - The manifest file cannot be found.
/// - There's an issue executing the metadata command.
/// - Any other error occurs during the metadata retrieval process.
pub fn get_target_dir(manifest_path: &Path) -> Result<PathBuf, cargo_metadata::Error> {
    Ok(cargo_metadata::MetadataCommand::new()
        .manifest_path(manifest_path)
        .exec()?
        .target_directory
        .to_path_buf()
        .into_std_path_buf()
        .join("loam"))
}

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
            .and_then(|subcontract| subcontract.as_object())
            .and_then(|subcontract_object| subcontract_object.get(&key.to_string()))
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

/// Retrieves all dependencies for the given manifest path.
///
/// This function executes `cargo tree` to get the dependency tree and processes the output
/// to return a vector of `Package` structs representing all dependencies, including the root package.
///
/// # Arguments
///
/// * `manifest_path` - A reference to the Path of the Cargo.toml file.
///
/// # Returns
///
/// A `Result` containing a `Vec<Package>` on success, or an `Error` on failure.
///
/// # Errors
///
/// This function will return an error in the following situations:
/// - If the metadata command fails to execute
/// - If the root package is not found in the metadata
/// - If the parent directory of the manifest path cannot be determined
/// - If the `cargo tree` command fails to execute
///
/// # Panics
///
/// This function may panic in the following situations:
/// - If the output of `cargo tree` contains invalid UTF-8 characters
/// - If the parsing of package names and versions from the `cargo tree` output fails
pub fn all(manifest_path: &Path) -> Result<Vec<Package>, Error> {
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
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let output = Command::new(cargo)
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
            res.cloned()
        })
        .collect::<Vec<_>>();
    res.push(p.clone());
    Ok(res)
}

#[must_use]
pub fn out_dir(target_dir: &Path, name: &str) -> PathBuf {
    target_dir.join("loam").join(name.replace('-', "_"))
}

pub enum DepKind {
    Subcontract,
    Contract,
}

impl Display for DepKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DepKind::Subcontract => write!(f, "subcontract"),
            DepKind::Contract => write!(f, "contract"),
        }
    }
}

/// Retrieves a list of source and output paths for dependencies of a specified kind.
///
/// # Arguments
///
/// * `manifest_path` - The path to the Cargo.toml manifest file.
/// * `kind` - The kind of dependency to filter.
///
/// # Returns
///
/// A `Result` containing a vector of tuples, where each tuple contains:
/// - The path to the dependency's `lib.rs` file
/// - The output directory for the dependency
///
/// # Errors
///
/// This function can return an error in the following cases:
/// - If there's an issue reading or parsing the manifest file
/// - If a dependency's manifest path doesn't have a parent directory
/// - If there are any issues accessing or processing the dependency information
pub fn loam(manifest_path: &Path, kind: &DepKind) -> Result<Vec<(Utf8PathBuf, PathBuf)>, Error> {
    all(manifest_path)?
        .into_iter()
        .filter(|p| p.is_dep(kind) || p.manifest_path == manifest_path)
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
/// Retrieves subcontract dependencies from the given manifest path.
///
/// # Arguments
///
/// * `manifest_path` - The path to the Cargo.toml manifest file.
///
/// # Returns
///
/// A `Result` containing a vector of tuples, where each tuple contains:
/// - A `Utf8PathBuf` representing the package name and version.
/// - A `PathBuf` representing the path to the package.
///
/// # Errors
///
/// This function may return an error if:
/// - The manifest file cannot be read or parsed.
/// - There are issues accessing or processing dependency information.
/// - Any other error occurs during the dependency resolution process.
pub fn subcontract_paths(manifest_path: &Path) -> Result<Vec<(Utf8PathBuf, PathBuf)>, Error> {
    loam(manifest_path, &DepKind::Subcontract)
}

/// Retrieves a list of contract dependencies for a given manifest path.
///
/// This function filters all dependencies of the package specified by the manifest path,
/// returning only those that are of the Contract kind and are not the package itself.
///
/// # Arguments
///
/// * `manifest_path` - A Path to the Cargo.toml manifest file.
///
/// # Returns
///
/// A Result containing a Vec of Package structs representing the contract dependencies,
/// or an Error if the operation fails.
///
/// # Errors
///
/// This function will return an Error if:
/// * There's an issue reading or parsing the manifest file.
/// * There's a problem retrieving the dependencies.
pub fn contract(manifest_path: &Path) -> Result<Vec<Package>, Error> {
    Ok(all(manifest_path)?
        .into_iter()
        .filter(|p| p.is_dep(&DepKind::Contract) && p.manifest_path != manifest_path)
        .collect())
}

/// Retrieves a list of subcontract dependencies for a given manifest path.
///
/// This function filters all dependencies of the package specified by the manifest path,
/// returning only those that are of the Contract kind and are not the package itself.
///
/// # Arguments
///
/// * `manifest_path` - A Path to the Cargo.toml manifest file.
///
/// # Returns
///
/// A Result containing a Vec of Package structs representing the contract dependencies,
/// or an Error if the operation fails.
///
/// # Errors
///
/// This function will return an Error if:
/// * There's an issue reading or parsing the manifest file.
/// * There's a problem retrieving the dependencies.
pub fn subcontract(manifest_path: &Path) -> Result<Vec<Package>, Error> {
    Ok(all(manifest_path)?
        .into_iter()
        .filter(|p| p.is_dep(&DepKind::Subcontract))
        .collect())
}

/// Constructs a workspace from a list of packages, sorting them topologically based on their contract dependencies.
///
/// This function creates a dependency graph of the provided packages and their contract dependencies,
/// then returns a topologically sorted list of these packages.
///
/// # Arguments
///
/// * `packages` - A slice of Package structs to process.
///
/// # Returns
///
/// A Result containing a Vec of Package structs representing the sorted workspace,
/// or an Error if the operation fails.
///
/// # Errors
///
/// This function will return an Error if:
/// * There's an issue retrieving contract dependencies for any of the packages.
/// * The dependency graph contains cycles, making topological sorting impossible.
pub fn get_workspace(packages: &[Package]) -> Result<Vec<Package>, Error> {
    let mut graph: TopologicalSort<PackageId> = TopologicalSort::new();
    for p in packages {
        let contract_deps = contract(&p.manifest_path.clone().into_std_path_buf())?;
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
        let deps = all(&manifest_path).unwrap();
        println!("{deps:#?}\n{}", deps.len());
        let deps = subcontract_paths(&manifest_path).unwrap();
        println!("{deps:#?}\n{}", deps.len());
    }
}

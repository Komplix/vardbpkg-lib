use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Represents a package in the Gentoo vardb.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct VarDbPkg {
    pub category: String,
    pub package: String,
    pub version: String,
    pub buildtime: String,
    pub description: String,
    pub homepage: String,
    pub iuse: String,
    pub keywords: String,
    pub license: String,
    pub rdepend: String,
    pub repository: String,
    pub slot: String,
    pub usepkg: String,
    pub eapi: String,
    pub binpkgmd5: String,
}

/// Parses the entire vardb at the given path.
/// Typically this is `/var/db/pkg`.
pub fn parse_vardb<P: AsRef<Path>>(path: P) -> Vec<VarDbPkg> {
    let mut packages = Vec::new();
    let root = path.as_ref();

    if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.flatten() {
            let category_path = entry.path();
            if category_path.is_dir() {
                let category_name = entry.file_name().to_string_lossy().into_owned();

                if let Ok(pkg_entries) = fs::read_dir(&category_path) {
                    for pkg_entry in pkg_entries.flatten() {
                        let pkg_path = pkg_entry.path();
                        if pkg_path.is_dir() {
                            let pkg_dir_name = pkg_entry.file_name().to_string_lossy().into_owned();
                            if let Some(pkg_info) =
                                parse_package_dir(&category_name, &pkg_dir_name, &pkg_path)
                            {
                                packages.push(pkg_info);
                            }
                        }
                    }
                }
            }
        }
    }

    packages
}

fn parse_package_dir(category: &str, dir_name: &str, path: &Path) -> Option<VarDbPkg> {
    let (package_name, version) = split_package_version(dir_name);

    let mut pkg = VarDbPkg {
        category: category.to_string(),
        package: package_name,
        version,
        ..Default::default()
    };

    pkg.buildtime = read_first_line(path.join("BUILD_TIME")).unwrap_or_default();
    pkg.description = read_first_line(path.join("DESCRIPTION")).unwrap_or_default();
    pkg.homepage = read_first_line(path.join("HOMEPAGE")).unwrap_or_default();
    pkg.iuse = read_first_line(path.join("IUSE")).unwrap_or_default();
    pkg.keywords = read_first_line(path.join("KEYWORDS")).unwrap_or_default();
    pkg.license = read_first_line(path.join("LICENSE")).unwrap_or_default();
    pkg.rdepend = read_first_line(path.join("RDEPEND")).unwrap_or_default();
    pkg.repository = read_first_line(path.join("repository")).unwrap_or_default();
    pkg.slot = read_first_line(path.join("SLOT")).unwrap_or_default();
    pkg.usepkg = read_first_line(path.join("USE")).unwrap_or_default();
    pkg.eapi = read_first_line(path.join("EAPI")).unwrap_or_default();
    pkg.binpkgmd5 = read_first_line(path.join("BINPKGMD5")).unwrap_or_default();

    Some(pkg)
}

/// Splits a directory name into package name and version.
/// Gentoo package directories are named as `package-version`.
fn split_package_version(dir_name: &str) -> (String, String) {
    let parts: Vec<&str> = dir_name.split('-').collect();

    for i in 1..parts.len() {
        if let Some(first_char) = parts[i].chars().next() {
            if first_char.is_ascii_digit() {
                let package_name = parts[..i].join("-");
                let version = parts[i..].join("-");
                return (package_name, version);
            }
        }
    }

    (dir_name.to_string(), String::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_split_package_version() {
        assert_eq!(
            split_package_version("amanda-0-r2"),
            ("amanda".to_string(), "0-r2".to_string())
        );
        assert_eq!(
            split_package_version("gcc-11.2.0"),
            ("gcc".to_string(), "11.2.0".to_string())
        );
        assert_eq!(
            split_package_version("my-pkg-name-1.2.3-r1"),
            ("my-pkg-name".to_string(), "1.2.3-r1".to_string())
        );
        assert_eq!(
            split_package_version("noversion"),
            ("noversion".to_string(), "".to_string())
        );
    }

    #[test]
    fn test_read_first_line() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = fs::File::create(&file_path).unwrap();
        writeln!(file, "  first line  ").unwrap();
        writeln!(file, "second line").unwrap();

        assert_eq!(read_first_line(&file_path), Some("first line".to_string()));
        assert_eq!(read_first_line(dir.path().join("nonexistent")), None);
    }
}

/// Reads the first line of a file and trims it.
fn read_first_line<P: AsRef<Path>>(path: P) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .and_then(|content| content.lines().next().map(|s| s.trim().to_string()))
}

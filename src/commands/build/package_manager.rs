use glob::glob;
use std::{env, fs};
use std::path::Path;
use serde_derive::{Serialize};
use serde_json::Value;
use serde_xml_rs::Error;
use crate::doctavious_error::Result as DoctaviousResult;
use crate::commands::build::csproj::CSProj;

// TODO: should I flip relationship and instead have Projects with supported package managers?

// TODO: could add PDM and Anaconda (Python)
#[non_exhaustive]
#[derive(Serialize)]
pub enum PackageManager {
    Cargo,
    Go,
    NPM,
    Nuget,
    Poetry,
    PIP,
    PNPM,
    RubyGems,
    Yarn,
}

// TODO: do we need to support non-lock files?
#[derive(Serialize)]
pub struct PackageManagerInfo {
    pub name: &'static str,
    pub install_command: &'static str,

    // TODO: do we want to change to known_project_files?
    // we would also bring the concept of a exact known file or something like glob
    // for cases in which we dont have a known file ex: dotnet .csproj files
    // pub manifests: &'a [&'static str],
    // pub project_files: &'a [ProjectFile],

    // TODO: multiple files?
    pub lock_file: &'static str
}

impl<'a> PackageManager {
    pub const ALL: &'a [PackageManager] = &[
        PackageManager::Cargo,
        PackageManager::Go,
        PackageManager::NPM,
        PackageManager::Poetry,
        PackageManager::PIP,
        PackageManager::PNPM,
        PackageManager::RubyGems,
        PackageManager::Yarn,
    ];

    pub const fn info(&self) -> PackageManagerInfo {
        match self {
            PackageManager::Cargo => {
                PackageManagerInfo {
                    name: "cargo",
                    install_command: "cargo add",
                    // manifests: &["package.json"],
                    // project_files: &[ProjectFile::CargoToml],
                    lock_file: "Cargo.lock",
                }
            },
            PackageManager::Go => {
                PackageManagerInfo {
                    name: "go",
                    install_command: "go get",
                    // manifests: &["package.json"],
                    // project_files: &[ProjectFile::GoMod],
                    // TODO: not sure this is appropriate for a lock file
                    lock_file: "go.sum",
                }
            },
            PackageManager::NPM => {
                PackageManagerInfo {
                    name: "npm",
                    install_command: "npm install",
                    // manifests: &["package.json"],
                    // project_files: &[ProjectFile::PackageJson],
                    lock_file: "package-lock.json",
                }
            }
            PackageManager::Nuget => {
                PackageManagerInfo {
                    name: "nuget",
                    install_command: "dotnet add",
                    // manifests: &["package.json"],
                    // project_files: &[ProjectFile::CSProj],
                    lock_file: "packages.lock.json",
                }
            }
            PackageManager::Poetry => {
                PackageManagerInfo {
                    name: "poetry",
                    install_command: "poetry install",
                    // manifests: &["pyproject.toml"],
                    // project_files: &[ProjectFile::PyProject],
                    lock_file: "poetry.lock",
                }
            }
            PackageManager::PIP => {
                PackageManagerInfo {
                    name: "pip",
                    install_command: "pip install",
                    // manifests: &["pipfile", "requirements.txt"],
                    // project_files: &[ProjectFile::PipFile, ProjectFile::RequirementsTxt],
                    lock_file: "pipfile.lock",
                }
            }
            PackageManager::PNPM => {
                PackageManagerInfo {
                    name: "pnpm",
                    install_command: "pnpm install",
                    // manifests: &["package.json"],
                    // project_files: &[ProjectFile::PackageJson],
                    lock_file: "pnpm-lock.yaml",
                }
            }
            PackageManager::RubyGems => {
                PackageManagerInfo {
                    name: "rubygems",
                    install_command: "gem install",
                    // manifests: &["Gemfile"],
                    // project_files: &[ProjectFile::GemFile],
                    lock_file: "Gemfile.lock",
                }
            }
            PackageManager::Yarn => {
                PackageManagerInfo {
                    name: "yarn",
                    install_command: "yarn install",
                    // manifests: &["package.json"],
                    // project_files: &[ProjectFile::PackageJson],
                    lock_file: "yarn.lock",
                }
            }
        }
    }

    // pub fn has_dependency(&self, dependency: &str) -> bool {
    //     for p in self.info().project_files {
    //         let found = p.has_dependency(dependency);
    //         // TODO: do we want to log error that file could not be read? Do we want to separate
    //         // file doesnt exist and file cannt be read?
    //         if found.is_ok() {
    //             return true
    //         }
    //     }
    //     false
    //     // match self {
    //     //     PackageManager::NPM => {
    //     //         for manifest in &self.info().manifests {
    //     //             // TODO: read_config to Value
    //     //             let root: Value = serde_json::from_str(manifest)?;
    //     //
    //     //             if root.get("dependencies").and_then(dependency).is_some() {
    //     //                 return true
    //     //             }
    //     //         }
    //     //         false
    //     //     }
    //     //     PackageManager::Poetry => {
    //     //         for manifest in &self.info().manifests {
    //     //             // TODO: read_config to Value
    //     //             let root: Value = serde_json::from_str(manifest)?;
    //     //             if root.get("tool.poetry.dependencies").and_then(dependency).is_some() {
    //     //                 return true
    //     //             }
    //     //         }
    //     //         false
    //     //     }
    //     //     PackageManager::PIP => {
    //     //         // TODO: handle requirements.txt and Pipfile
    //     //     }
    //     //     PackageManager::PNPM => {
    //     //         for manifest in &self.info().manifests {
    //     //             // TODO: read_config to Value
    //     //             let root: Value = serde_json::from_str(manifest)?;
    //     //             // might be to do these individual lookup
    //     //             if root.get("tool.poetry.dependencies").and_then(dependency).is_some() {
    //     //                 return true
    //     //             }
    //     //         }
    //     //         false
    //     //     }
    //     //     PackageManager::RubyGems => {
    //     //         // will a contains be good enough?
    //     //     }
    //     //     PackageManager::Yarn => {
    //     //         for manifest in &self.info().manifests {
    //     //             // TODO: read_config to Value
    //     //             let root: Value = serde_json::from_str(manifest)?;
    //     //             if root.get("dependencies").and_then(dependency).is_some() {
    //     //                 return true
    //     //             }
    //     //         }
    //     //         false
    //     //     }
    //     // }
    // }
}





// impl FromStr for PackageManager {
//     type Err = String;
//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         match s {
//             "cargo" => Ok(PackageManager::Cargo),
//             "pnpm" => Ok(PackageManager::Pnpm),
//             "yarn" => Ok(PackageManager::Yarn),
//             "npm" => Ok(PackageManager::Npm),
//             _ => Err("Invalid package manager".to_string()),
//         }
//     }
// }


// JavaScript
// npm - package-lock.json, package.json
// yarn - yarn.lock, package.json
// pnpm - pnpm-lock.yaml
// pub struct Npm {
//     pub info: PackageManagerInfo
// }
//
// impl Default for Npm {
//     fn default() -> Self {
//         Self {
//             info: PackageManagerInfo {
//                 name: "npm",
//                 install_command: "npm install",
//                 manifests: vec!["package.json"],
//                 lock_file: "package-lock.json",
//             },
//         }
//     }
// }
//
// pub struct Yarn {
//     pub info: PackageManagerInfo
// }
//
// impl Default for Yarn {
//     fn default() -> Self {
//         Self {
//             info: PackageManagerInfo {
//                 name: "yarn",
//                 install_command: "yarn install",
//                 manifests: vec!["package.json"],
//                 lock_file: "yarn.lock",
//             },
//         }
//     }
// }
//
// pub struct Pnpm {
//     pub info: PackageManagerInfo
// }
//
// impl Default for Pnpm {
//     fn default() -> Self {
//         Self {
//             info: PackageManagerInfo {
//                 name: "pnpm",
//                 install_command: "pnpm install",
//                 manifests: vec!["package.json"],
//                 lock_file: "pnpm-lock.yaml",
//             },
//         }
//     }
// }
//
// pub struct Pip {
//     pub info: PackageManagerInfo
// }
//
// impl Default for Pip {
//     fn default() -> Self {
//         Self {
//             info: PackageManagerInfo {
//                 name: "pip",
//                 install_command: "pip install",
//                 manifests: vec!["pipfile", "requirements.txt"],
//                 lock_file: "pipfile.lock",
//             },
//         }
//     }
// }
//
// pub struct Poetry {
//     pub info: PackageManagerInfo
// }
//
// impl Default for Poetry {
//     fn default() -> Self {
//         Self {
//             info: PackageManagerInfo {
//                 name: "poetry",
//                 install_command: "poetry install",
//                 manifests: vec!["pyproject.toml"],
//                 lock_file: "poetry.lock",
//             },
//         }
//     }
// }
//
// pub struct RubyGems {
//     pub info: PackageManagerInfo
// }
//
// impl Default for RubyGems {
//     fn default() -> Self {
//         Self {
//             info: PackageManagerInfo {
//                 name: "rubygems",
//                 install_command: "",
//                 manifests: vec!["Gemfile"],
//                 lock_file: "Gemfile.lock",
//             },
//         }
//     }
// }

// JavaScript
// npm - package-lock.json, package.json
// yarn - yarn.lock, package.json
// pnpm - pnpm-lock.yaml

// Python
// pip - requirements.txt, pipfile.lock, pipfile, setup.py
// poetry - poetry.lock, pyproject.toml

// Ruby
// Gems - Gemfile.lock, Gemfile

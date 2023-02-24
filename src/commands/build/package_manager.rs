use glob::glob;
use std::{env, fs};
use std::path::Path;
use serde_derive::{Serialize, Deserialize};
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
pub struct PackageManagerInfo<'a> {
    pub name: &'static str,
    pub install_command: &'static str,

    // TODO: do we want to change to known_project_files?
    // we would also bring the concept of a exact known file or something like glob
    // for cases in which we dont have a known file ex: dotnet .csproj files
    // pub manifests: &'a [&'static str],
    pub project_files: &'a [ProjectFile],
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
                    project_files: &[ProjectFile::Cargo],
                    lock_file: "Cargo.lock",
                }
            },
            PackageManager::Go => {
                PackageManagerInfo {
                    name: "go",
                    install_command: "go get",
                    // manifests: &["package.json"],
                    project_files: &[ProjectFile::GoMod],
                    // TODO: not sure this is appropriate for a lock file
                    lock_file: "go.sum",
                }
            },
            PackageManager::NPM => {
                PackageManagerInfo {
                    name: "npm",
                    install_command: "npm install",
                    // manifests: &["package.json"],
                    project_files: &[ProjectFile::PackageJson],
                    lock_file: "package-lock.json",
                }
            }
            PackageManager::Nuget => {
                PackageManagerInfo {
                    name: "nuget",
                    install_command: "dotnet add",
                    // manifests: &["package.json"],
                    project_files: &[ProjectFile::CSProj],
                    lock_file: "packages.lock.json",
                }
            }
            PackageManager::Poetry => {
                PackageManagerInfo {
                    name: "poetry",
                    install_command: "poetry install",
                    // manifests: &["pyproject.toml"],
                    project_files: &[ProjectFile::PyProject],
                    lock_file: "poetry.lock",
                }
            }
            PackageManager::PIP => {
                PackageManagerInfo {
                    name: "pip",
                    install_command: "pip install",
                    // manifests: &["pipfile", "requirements.txt"],
                    project_files: &[ProjectFile::PipFile, ProjectFile::RequirementsTxt],
                    lock_file: "pipfile.lock",
                }
            }
            PackageManager::PNPM => {
                PackageManagerInfo {
                    name: "pnpm",
                    install_command: "pnpm install",
                    // manifests: &["package.json"],
                    project_files: &[ProjectFile::PackageJson],
                    lock_file: "pnpm-lock.yaml",
                }
            }
            PackageManager::RubyGems => {
                PackageManagerInfo {
                    name: "rubygems",
                    install_command: "gem install",
                    // manifests: &["Gemfile"],
                    project_files: &[ProjectFile::GemFile],
                    lock_file: "Gemfile.lock",
                }
            }
            PackageManager::Yarn => {
                PackageManagerInfo {
                    name: "yarn",
                    install_command: "yarn install",
                    // manifests: &["package.json"],
                    project_files: &[ProjectFile::PackageJson],
                    lock_file: "yarn.lock",
                }
            }
        }
    }

    pub fn has_dependency(&self, dependency: &str) -> bool {
        for p in self.info().project_files {
            let found = p.has_dependency(dependency);
            // TODO: do we want to log error that file could not be read? Do we want to separate
            // file doesnt exist and file cannt be read?
            if found.is_ok() {
                return true
            }
        }
        false
        // match self {
        //     PackageManager::NPM => {
        //         for manifest in &self.info().manifests {
        //             // TODO: read_config to Value
        //             let root: Value = serde_json::from_str(manifest)?;
        //
        //             if root.get("dependencies").and_then(dependency).is_some() {
        //                 return true
        //             }
        //         }
        //         false
        //     }
        //     PackageManager::Poetry => {
        //         for manifest in &self.info().manifests {
        //             // TODO: read_config to Value
        //             let root: Value = serde_json::from_str(manifest)?;
        //             if root.get("tool.poetry.dependencies").and_then(dependency).is_some() {
        //                 return true
        //             }
        //         }
        //         false
        //     }
        //     PackageManager::PIP => {
        //         // TODO: handle requirements.txt and Pipfile
        //     }
        //     PackageManager::PNPM => {
        //         for manifest in &self.info().manifests {
        //             // TODO: read_config to Value
        //             let root: Value = serde_json::from_str(manifest)?;
        //             // might be to do these individual lookup
        //             if root.get("tool.poetry.dependencies").and_then(dependency).is_some() {
        //                 return true
        //             }
        //         }
        //         false
        //     }
        //     PackageManager::RubyGems => {
        //         // will a contains be good enough?
        //     }
        //     PackageManager::Yarn => {
        //         for manifest in &self.info().manifests {
        //             // TODO: read_config to Value
        //             let root: Value = serde_json::from_str(manifest)?;
        //             if root.get("dependencies").and_then(dependency).is_some() {
        //                 return true
        //             }
        //         }
        //         false
        //     }
        // }
    }
}

// Manifest
#[non_exhaustive]
#[derive(Serialize)]
pub enum ProjectFile {
    Cargo,
    CSProj,
    GemFile,
    GoMod,
    PackageJson,
    PipFile,
    PyProject,
    RequirementsTxt,
}

impl ProjectFile {

    // pub fn file_name(&self) -> &str {
    //     match self {
    //         ProjectFile::GemFile => "Gemfile",
    //         ProjectFile::PackageJson => "package.json",
    //         ProjectFile::PipFile => "pipfile",
    //         ProjectFile::PyProject => "pyproject.toml",
    //         ProjectFile::RequirementsTxt => "requirements.txt"
    //     }
    // }

    pub fn has_dependency(&self, dependency: &str) -> DoctaviousResult<bool> {
        // let project_file_content = fs::read_to_string(self.file_name())?;
        let found = match self {
            ProjectFile::Cargo => {
                let project_file_content = fs::read_to_string("cargo.toml")?;
                let root: toml::Value = toml::from_str(project_file_content.as_str())?;
                // TODO: do we want to check dev-packages
                root.get("dependencies")
                    .and_then(|o| o.get(dependency))
                    .is_some()
            }
            ProjectFile::CSProj => {
                println!("hello there...");
                let mut has_dependency = false;
                match env::current_dir() {
                    Ok(p) => {
                        println!("path...{:?}", p);
                    }
                    Err(e) => {
                        println!("error getting cwd...{}", e);
                    }
                }

                let paths = fs::read_dir("./").unwrap();
                for path in paths {
                    println!("path name: {}", path.unwrap().path().display())
                }
                for file in glob("*")? {
                    println!("path...{:?}", file.unwrap());
                }

                for entry in glob("**/*.csproj")? {
                    if let Ok(entry) = entry {
                        println!("entry...{:?}", entry);
                        if let Some(path_str) = entry.to_str() {
                            // TODO: much rather deal with something like Value but not supported by serde_xml_rs
                            // Could move to xml2json/xmltojson or something like quick-xml serde
                            let content = fs::read_to_string(path_str)?;
                            let result: Result<CSProj, _> = serde_xml_rs::from_str(content.as_str());
                            // match result {
                            //     Ok(r) => {
                            //         println!("got project...{:?}", r);
                            //     }
                            //     Err(e) => {
                            //         println!("serde error...{:?}", e);
                            //     }
                            // }
                            if let Ok(build_proj) = result {
                                println!("build proj...{:?}", build_proj);
                                for item_group in build_proj.item_groups {
                                    // could also do item_group.package_references.unwrap_or_default()
                                    if let Some(package_references ) = item_group.package_references {
                                        for pkref in package_references {
                                            if dependency == pkref.include {
                                                has_dependency = true;
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                has_dependency
            }
            ProjectFile::GemFile => {
                let project_file_content = fs::read_to_string("Gemfile")?;
                project_file_content.contains(&format!("gem '{}'", dependency))
            }
            ProjectFile::GoMod => {
                todo!("implement")
            }
            ProjectFile::PackageJson => {
                let project_file_content = fs::read_to_string("package.json")?;
                let root: Value = serde_json::from_str(project_file_content.as_str())?;
                // TODO: do we want to check devDependencies
                root.get("dependencies")
                    .and_then(|o| o.get(dependency))
                    .is_some()
            }
            ProjectFile::PipFile => {
                let project_file_content = fs::read_to_string("pipfile")?;
                let root: toml::Value = toml::from_str(project_file_content.as_str())?;
                // TODO: do we want to check dev-packages
                root.get("packages")
                    .and_then(|o| o.get(dependency))
                    .is_some()
            }
            ProjectFile::PyProject => {
                let project_file_content = fs::read_to_string("pyproject.toml")?;
                let root: toml::Value = toml::from_str(project_file_content.as_str())?;
                // might be to do these individual lookup
                root.get("tool.poetry.dependencies")
                    .and_then(|o| o.get(dependency))
                    .is_some()
            }
            ProjectFile::RequirementsTxt => {
                let project_file_content = fs::read_to_string("requirements.txt")?;
                project_file_content.contains(&format!("{}==", dependency))
            }
        };

        Ok(found)
    }

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

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::{env, fs, io, panic};
    use std::panic::{RefUnwindSafe, UnwindSafe};
    use std::path::{Path, PathBuf};
    use std::sync::Mutex;
    use swc::atoms::once_cell;
    use once_cell::sync::Lazy;
    use tempfile::TempDir;
    use crate::commands::build::package_manager::ProjectFile;
    use crate::doctavious_error::DoctaviousError;
    use crate::doctavious_error::Result as DoctaviousResult;
    use std::io::Write;
    use serial_test::serial;

    static SERIAL_TEST: Lazy<Mutex<()>> = Lazy::new(Default::default);

    #[test]
    fn test_pyproject() {
        let content = r#"
[tool.poetry]
name = "poetry-demo"
version = "0.1.0"
description = ""
authors = ["Sébastien Eustace <sebastien@eustace.io>"]
readme = "README.md"
packages = [{include = "poetry_demo"}]

[tool.poetry.dependencies]
python = "^3.7"
        "#;
    }

    #[test]
    #[serial]
    fn test_csproj() -> DoctaviousResult<()> {
        let content = r#"
<Project Sdk="Microsoft.NET.Sdk">

  <PropertyGroup>
    <TargetFramework>net7.0</TargetFramework>
    <ImplicitUsings>enable</ImplicitUsings>
    <Nullable>enable</Nullable>
    <OutputType>Exe</OutputType>
    <ServerGarbageCollection>true</ServerGarbageCollection>
  </PropertyGroup>

  <ItemGroup>
    <PackageReference Include="Microsoft.Extensions.Hosting" Version="7.0.0" />
    <PackageReference Include="Microsoft.Extensions.Logging.Console" Version="7.0.0" />
    <PackageReference Include="Microsoft.Orleans.Server" Version="7.0.0" />
    <PackageReference Include="Microsoft.Orleans.Streaming" Version="7.0.0" />
  </ItemGroup>

  <ItemGroup>
    <ProjectReference Include="..\ChatRoom.Common\ChatRoom.Common.csproj" />
  </ItemGroup>

</Project>"#;

        let tmp_dir = TempDir::new().unwrap();
        let file_path = tmp_dir.path().join("docs.csproj");
        let mut tmp_file = File::create(file_path).unwrap();
        writeln!(tmp_file, "{}", content)?;

        // println!("{:?}", tmp_dir);
        // println!("cur dir is  {:?}", env::current_dir()?);
        let a = with_dir(&tmp_dir, || {
            match env::current_dir() {
                Ok(p) => {
                    println!("path...{:?}", p);
                }
                Err(e) => {
                    println!("error with cwd...{}", e);
                }
            }

            let paths = fs::read_dir(&tmp_dir).unwrap();
            for path in paths {
                println!("path name: {}", path.unwrap().path().display())
            }

            let found = ProjectFile::CSProj.has_dependency("Microsoft.Orleans.Server").unwrap();
            println!("dependency found: {}", found);
            assert!(found);
            Ok(())
        });

        match a {
            Ok(_) => {}
            Err(e) => {
                println!("error was {}", e);
            }
        }



        // let found = ProjectFile::CSProj.has_dependency("Microsoft.Orleans.Server")?;
        // assert!(found);

        Ok(())
    }

    // with_directory(path, || { closure })



    pub fn with_dir<P, F, R>(path: &P, closure: F) -> io::Result<R>
        where
            P: AsRef<Path>,
            F: Fn() -> io::Result<R> + UnwindSafe + RefUnwindSafe,
    {
        let guard = SERIAL_TEST.lock().unwrap();
        let original_dir = env::current_dir()?;
        match env::set_current_dir(path) {
            Ok(_) => {
                println!("success");
            }
            Err(e) => {
                println!("error...{:?}", e);
            }
        }

        // println!("current...{:?}", env::current_dir()?);
        let a = match panic::catch_unwind(closure) {
            Ok(result) => {
                println!("original dir...{:?}", original_dir);
                env::set_current_dir(original_dir)?;
                // drop(path); // not sure if we need do drop this here
                result
            }
            Err(err) => {
                println!("error occurred original dir...{:?}", original_dir);
                env::set_current_dir(original_dir)?;
                // drop(path);
                drop(guard);
                panic::resume_unwind(err);
            }
        };
        a
    }
}

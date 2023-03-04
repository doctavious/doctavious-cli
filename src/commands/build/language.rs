use std::path::{Path, PathBuf};
use glob::glob;
use serde_derive::Serialize;
use crate::commands::build::package_manager::{PackageManager, PackageManagerInfo};
use crate::commands::build::projects::project_file::{Proj, ProjectFile};

// TODO: We might need to determine python path in order to do python builds

#[non_exhaustive]
#[derive(Serialize)]
pub enum Language {
    DotNet,
    Go,
    Javascript,
    Python,
    Ruby,
    Rust
}

impl Language {

    // pub fn get_projects(&self) -> Vec<Proj> {
    //     match self {
    //         Language::DotNet => {
    //             let glob_result = glob("**/*.csproj");
    //             match glob_result {
    //                 Ok(paths) => {
    //                     let mut projects = Vec::new();
    //                     for path in paths {
    //                         if let Ok(path) = path {
    //                             let project = Proj::new(path, ProjectFile::CSProj);
    //                             match project {
    //                                 Ok(p) => projects.push(p),
    //                                 Err(_) => {
    //                                     // TODO: print unable to read path
    //                                 }
    //                             }
    //
    //                         } else {
    //                             // TODO: log
    //                         }
    //                     }
    //                     projects
    //                 },
    //                 Err(e) => {
    //                     // TODO: log
    //                     vec![]
    //                 }
    //             }
    //         },
    //         Language::Go => vec![Proj::new(PathBuf::from("go.mod"), ProjectFile::GoMod)],
    //         Language::Javascript => vec![Proj::new(PathBuf::from("package.json"), ProjectFile::PackageJson],
    //         Language::Python => vec![
    //             PathBuf::from("pipfile"),
    //             PathBuf::from("pyproject.toml"),
    //             PathBuf::from("requirements.txt")
    //         ],
    //         Language::Ruby => vec![PathBuf::from("Gemfile")],
    //         Language::Rust => vec![PathBuf::from("cargo.toml")]
    //     }
    // }

    // pub fn get_project_paths(&self) -> Vec<PathBuf> {
    //     match self {
    //         Language::DotNet => {
    //             let glob_result = glob("**/*.csproj");
    //             match glob_result {
    //                 Ok(paths) => {
    //                     paths.into_iter().filter_map(|p| p.ok()).collect()
    //                 },
    //                 Err(e) => {
    //                     // TODO: log
    //                     vec![]
    //                 }
    //             }
    //         },
    //         Language::Go => vec![PathBuf::from("go.mod")],
    //         Language::Javascript => vec![PathBuf::from("package.json")],
    //         Language::Python => vec![
    //             PathBuf::from("pipfile"),
    //             PathBuf::from("pyproject.toml"),
    //             PathBuf::from("requirements.txt")
    //         ],
    //         Language::Ruby => vec![PathBuf::from("Gemfile")],
    //         Language::Rust => vec![PathBuf::from("cargo.toml")]
    //     }
    // }

    // pub fn get_project_files(&self) -> Vec<Proj> {
    //     match self {
    //         Language::DotNet => {
    //             let project_file_content = fs::read_to_string("cargo.toml")?;
    //         },
    //         Language::Go => &[ProjectFile::GoMod],
    //         Language::Javascript => &[ProjectFile::PackageJson],
    //         Language::Python => &[ProjectFile::PyProject, ProjectFile::PipFile, ProjectFile::RequirementsTxt],
    //         Language::Ruby => &[ProjectFile::GemFile],
    //         Language::Rust => &[ProjectFile::CargoToml]
    //     }
    // }

    pub const fn project_files(&self) -> &[ProjectFile] {
        match self {
            Language::DotNet => &[ProjectFile::CSProj],
            Language::Go => &[ProjectFile::GoMod],
            Language::Javascript => &[ProjectFile::PackageJson],
            Language::Python => &[ProjectFile::PyProject, ProjectFile::PipFile, ProjectFile::RequirementsTxt],
            Language::Ruby => &[ProjectFile::GemFile],
            Language::Rust => &[ProjectFile::CargoToml]
        }
    }

    pub const fn get_package_managers(&self) -> &[PackageManager] {
        match self {
            Language::DotNet => &[PackageManager::Nuget],
            Language::Go => &[PackageManager::Go],
            Language::Javascript => &[PackageManager::NPM, PackageManager::PNPM, PackageManager::Yarn],
            Language::Python => &[PackageManager::Poetry, PackageManager::PIP],
            Language::Ruby => &[PackageManager::RubyGems],
            Language::Rust => &[PackageManager::Cargo]
        }
    }

}

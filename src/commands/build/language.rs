use serde_derive::Serialize;
use crate::commands::build::package_manager::{PackageManager, PackageManagerInfo};

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

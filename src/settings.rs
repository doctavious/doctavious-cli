use crate::constants::{DEFAULT_ADR_DIR, DEFAULT_CONFIG_NAME, DEFAULT_TIL_DIR};
use crate::doctavious_error::Result;
use crate::file_structure::FileStructure;
// TODO: fix this
use crate::commands::githooks::githooks::Hook;
use lazy_static::lazy_static;
use std::fs;
use std::str;

use crate::commands::changelog::CommitParser;
use crate::markup_format::MarkupFormat;
use regex::Regex;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

lazy_static! {
    // TODO: doctavious config will live in project directory
    // do we also want a default settings file
    pub static ref SETTINGS_FILE: PathBuf = PathBuf::from(DEFAULT_CONFIG_NAME);

    // TODO: not sure this buys us anything.
    // just have a parse method on Settings that takes in a string/pathbuf?
    pub static ref SETTINGS: Settings = {
        match load_settings() {
            Ok(settings) => settings,
            Err(e) => {
                if Path::new(SETTINGS_FILE.as_path()).exists() {
                    eprintln!(
                        "Error when parsing {}, fallback to default settings. Error: {:?}\n",
                        SETTINGS_FILE.as_path().display(),
                        e
                    );
                }
                Default::default()
            }
        }
    };
}

// TODO: should this include output?
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Settings {
    pub template_extension: Option<MarkupFormat>,

    #[serde(rename(serialize = "adr"))]
    #[serde(alias = "adr")]
    pub adr_settings: Option<AdrSettings>,

    #[serde(rename(serialize = "rfd"))]
    #[serde(alias = "rfd")]
    pub rfd_settings: Option<RFDSettings>,

    #[serde(rename(serialize = "til"))]
    #[serde(alias = "til")]
    pub til_settings: Option<TilSettings>,

    #[serde(rename(serialize = "changelog"))]
    #[serde(alias = "changelog")]
    pub changelog_settings: Option<ChangelogSettings>,

    #[serde(rename(serialize = "githook"))]
    #[serde(alias = "githook")]
    pub githook_settings: Option<ChangelogSettings>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ChangelogSettings {
    pub header: Option<String>,
    pub body: String,
    pub footer: Option<String>,
    pub trim: bool,
    pub git: ChangelogGitSettings,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ChangelogGitSettings {
    pub conventional_commits: bool,
    pub commit_parsers: Option<Vec<CommitParser>>,
    pub filter_commits: bool,
    pub tag_pattern: Option<String>,
    #[serde(with = "serde_regex", default)]
    /// Regex to skip matched tags.
    pub skip_tags: Option<Regex>,
    // https://github.com/orhun/git-cliff/issues/10
    // skip intermediate tags?
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct AdrSettings {
    pub dir: Option<String>,
    pub structure: Option<FileStructure>,
    pub template_extension: Option<MarkupFormat>,
    // TODO: custom date format

}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct RFDSettings {
    pub dir: Option<String>,
    pub structure: Option<FileStructure>,
    pub template_extension: Option<MarkupFormat>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct TilSettings {
    pub dir: Option<String>,
    pub template_extension: Option<MarkupFormat>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct GithookSettings {
    pub hooks: HashMap<String, Hook>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SnippetSettings {
    pub begin: String,
    pub end: String,
    pub extension: String,
    pub comment_prefix: String,
    pub template: String,
    pub sources: SnippetSource,
    pub output_dir: Option<String>,
    pub targets: Option<Vec<String>>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SnippetSource {
    pub repository: Option<String>,
    pub branch: Option<String>,
    pub starting_point: Option<String>,
    pub directory: Option<String>, // default to "."
    pub files: Vec<String>,
}

impl Settings {
    pub fn get_adr_dir(&self) -> &str {
        if let Some(settings) = &self.adr_settings {
            if let Some(dir) = &settings.dir {
                return dir;
            }
        }

        return DEFAULT_ADR_DIR;
    }

    pub fn get_adr_structure(&self) -> FileStructure {
        if let Some(settings) = &self.adr_settings {
            if let Some(structure) = settings.structure {
                return structure;
            }
        }

        return FileStructure::default();
    }

    pub fn get_adr_template_extension(
        &self,
        extension: Option<MarkupFormat>,
    ) -> MarkupFormat {
        if extension.is_some() {
            return extension.unwrap();
        }

        if let Some(settings) = &self.adr_settings {
            if let Some(template_extension) = settings.template_extension {
                return template_extension;
            }
        }

        if let Some(template_extension) = self.template_extension {
            return template_extension;
        }

        return MarkupFormat::default();
    }

    pub fn get_rfd_dir(&self) -> &str {
        if let Some(settings) = &self.rfd_settings {
            if let Some(dir) = &settings.dir {
                return dir;
            }
        }

        return DEFAULT_ADR_DIR;
    }

    pub fn get_rfd_structure(&self) -> FileStructure {
        if let Some(settings) = &self.rfd_settings {
            if let Some(structure) = settings.structure {
                return structure;
            }
        }

        return FileStructure::default();
    }

    pub fn get_rfd_template_extension(
        &self,
        extension: Option<MarkupFormat>,
    ) -> MarkupFormat {
        if extension.is_some() {
            return extension.unwrap();
        }

        if let Some(settings) = &self.rfd_settings {
            if let Some(template_extension) = settings.template_extension {
                return template_extension;
            }
        }

        if let Some(template_extension) = self.template_extension {
            return template_extension;
        }

        return MarkupFormat::default();
    }

    pub fn get_til_dir(&self) -> &str {
        if let Some(settings) = &self.til_settings {
            if let Some(dir) = &settings.dir {
                return dir;
            }
        }

        return DEFAULT_TIL_DIR;
    }

    // TODO: I might revert having this take in an extension and rather just have a function in til
    // that does and defers to settings
    pub fn get_til_template_extension(
        &self,
        extension: Option<MarkupFormat>,
    ) -> MarkupFormat {
        if extension.is_some() {
            return extension.unwrap();
        }

        if let Some(settings) = &self.til_settings {
            if let Some(template_extension) = settings.template_extension {
                return template_extension;
            }
        }

        if let Some(template_extension) = self.template_extension {
            return template_extension;
        }

        return MarkupFormat::default();
    }
}

pub(crate) fn load_settings() -> Result<Settings> {
    let bytes = std::fs::read(SETTINGS_FILE.as_path())?;
    let settings: Settings = toml::from_slice(&bytes)?;
    Ok(settings)
}

// outside of Settings because we dont want to initialize load given we are using lazy_static
// TODO: should this take in a mut writer, i.e., a mutable thing we call “writer”.
// Its type is impl std::io::Write
// so that its a bit easier to test?
pub(crate) fn persist_settings(settings: Settings) -> Result<()> {
    let content = toml::to_string(&settings)?;
    fs::write(SETTINGS_FILE.as_path(), content)?;

    Ok(())
}

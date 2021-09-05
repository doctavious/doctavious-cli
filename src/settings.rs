use crate::constants::{DEFAULT_ADR_DIR, DEFAULT_CONFIG_NAME, DEFAULT_TIL_DIR};
use crate::file_structure::FileStructure;
use crate::templates::TemplateExtension;
use lazy_static::lazy_static;
use std::fs;
use std::str;

use serde_derive::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

lazy_static! {
    // TODO: doctavious config will live in project directory
    // do we also want a default settings file
    pub static ref SETTINGS_FILE: PathBuf = PathBuf::from(DEFAULT_CONFIG_NAME);

    pub static ref SETTINGS: Settings = {
        match load_settings() {
            Ok(settings) => settings,
            Err(e) => {
                if Path::new(SETTINGS_FILE.as_path()).exists() {
                    eprintln!(
                        "Error when parsing {}, fallback to default settings. Error: {}\n",
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
    pub template_extension: Option<TemplateExtension>,

    #[serde(rename(serialize = "adr"))]
    #[serde(alias = "adr")]
    pub adr_settings: Option<AdrSettings>,

    #[serde(rename(serialize = "rfd"))]
    #[serde(alias = "rfd")]
    pub rfd_settings: Option<RFDSettings>,

    #[serde(rename(serialize = "til"))]
    #[serde(alias = "til")]
    pub til_settings: Option<TilSettings>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct AdrSettings {
    pub dir: Option<String>,
    pub structure: Option<FileStructure>,
    pub template_extension: Option<TemplateExtension>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct RFDSettings {
    pub dir: Option<String>,
    pub structure: Option<FileStructure>,
    pub template_extension: Option<TemplateExtension>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct TilSettings {
    pub dir: Option<String>,
    pub template_extension: Option<TemplateExtension>,
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

    pub fn get_adr_template_extension(&self, extension: Option<TemplateExtension>) -> TemplateExtension {
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

        return TemplateExtension::default();
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

    pub fn get_rfd_template_extension(&self, extension: Option<TemplateExtension>) -> TemplateExtension {
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

        return TemplateExtension::default();
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
    pub fn get_til_template_extension(&self, extension: Option<TemplateExtension>) -> TemplateExtension {
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

        return TemplateExtension::default();
    }
}

pub(crate) fn load_settings() -> Result<Settings, Box<dyn std::error::Error>> {
    let bytes = std::fs::read(SETTINGS_FILE.as_path())?;
    let settings: Settings = toml::from_slice(&bytes)?;
    Ok(settings)
}

// outside of Settings because we dont want to initialize load given we are using lazy_static
// TODO: should this take in a mut writer, i.e., a mutable thing we call “writer”.
// Its type is impl std::io::Write
// so that its a bit easier to test?
pub(crate) fn persist_settings(
    settings: Settings,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = toml::to_string(&settings)?;
    fs::write(SETTINGS_FILE.as_path(), content)?;

    Ok(())
}

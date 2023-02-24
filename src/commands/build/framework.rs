use std::fs;
use std::path::Path;

use serde::{Deserialize};
use serde_derive::Serialize;
use swc_ecma_ast::*;

use crate::commands::build::js_module::parse_js_module;
use crate::commands::build::language::Language;
use crate::doctavious_error::DoctaviousError;
use crate::DoctaviousResult;

#[derive(Serialize)]
pub struct FrameworkInfo {
    // id: String,

    /// Name of the framework
    ///
    /// # Examples
    /// Next.js
    pub name: &'static str,

    /// A URL to the official website of the framework
    ///
    /// # Examples
    /// https://nextjs.org
    pub website: Option<&'static str>,

    // TODO: could string be a glob?
    // TODO: does this really need to be an Option? How about just empty?
    /// List of potential config files
    pub configs: Option<Vec<&'static str>>,

    // TODO: maybe this should be language which then has package managers?
    // /// The file contains descriptive and functional metadata about a project
    // /// specifically dependencies
    // ///
    // /// # Examples
    // /// package.json, .csproj
    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub project_file: Option<&'static str>,
    pub language: Language,

    // /// Detectors used to find out the framework
    pub detection: FrameworkDetector,

    pub build: FrameworkBuildSettings,

}

impl FrameworkInfo {

    pub fn detected(&self) -> bool {
        let mut results = vec![];
        // let stop_on_first_found = FrameworkMatchingStrategy::Any == &self.detection.matching_strategy;
        for detection in &self.detection.detectors {
            let result = match detection {
                FrameworkDetectionItem::Config { content } => {
                    if let Some(configs) = &self.configs {
                        for config in configs {
                            if let Ok(file_content) = fs::read_to_string(config) {
                                if let Some(content) = content {
                                    if file_content.contains(content) {
                                        return true;
                                    }
                                    continue;
                                }
                                return true;
                            }
                        }
                    }
                    false
                }
                FrameworkDetectionItem::Package { dependency } => {
                    for pck_manager in self.language.get_package_managers() {
                        if pck_manager.has_dependency(dependency) {
                            return true;
                        }
                    }
                    false
                }
            };

            match &self.detection.matching_strategy {
                FrameworkMatchingStrategy::Every => {
                    results.push(result);
                }
                FrameworkMatchingStrategy::Any => {
                    if result {
                        results.push(result);
                        break;
                    }
                }
            }
        }

        // use std::convert::identity might be more idiomatic here
        results.iter().all(|&r| r)
    }

}

// TODO: rename to FrameworkDetection?
#[derive(Serialize)]
pub struct FrameworkDetector {
    pub matching_strategy: FrameworkMatchingStrategy,
    pub detectors: Vec<FrameworkDetectionItem>
}

#[derive(Serialize)]
pub enum FrameworkDetectionItem {
    // TODO: should path support glob or should we force individual items?

    // /// content - regex
    // Config { path: String, content: Option<String> },
    Config { content: Option<&'static str> },

    // TODO: change to Dependency with a name
    // /// content - regex
    // Package { path: String, content: String }
    Package { dependency: &'static str }
}

// TODO: better name. Enum?
// pub struct FrameworkDetectionItem {
//     /// A file path to detect.
//     path: Option<String>,
//     /// A matcher for the entire file.
//     /// @example "\"(dev)?(d|D)ependencies\":\\s*{[^}]*\"next\":\\s*\".+?\"[^}]*}"
//     matchContent: Option<String>,
//     // TODO: we need to support more than just package.json ex: .NET core
//     /// A matcher for a package specifically found in a "package.json" file.
//     /// @example "\"(dev)?(d|D)ependencies\":\\s*{[^}]*\"next\":\\s*\".+?\"[^}]*}"
//     matchPackage: Option<String>
// }


// TODO: change name?
#[derive(Serialize)]
pub enum FrameworkMatchingStrategy {
    /// Strategy that requires all detectors must match
    Every, // TODO: change to all

    /// Strategy where one match causes the framework to be detected
    Any
}




#[derive(Serialize)]
pub struct FrameworkBuildSettings {
    pub command: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_args: Option<FrameworkBuildArgs>,
    pub output_directory: &'static str
}

#[derive(Serialize)]
pub struct FrameworkBuildArgs {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<FrameworkBuildArg>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<FrameworkBuildArg>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<FrameworkBuildArg>
}

#[derive(Serialize)]
pub enum FrameworkBuildArg {
    // TODO: include named arguments
    /// 0-based index of argument and default value
    Arg(i8, Option<&'static str>),
    // TODO: do we care short or long? how about use vec/array?
    Option { short: &'static str, long: &'static str }
}


pub trait FrameworkSupport {

    fn get_info(&self) -> &FrameworkInfo;

    fn get_output_dir(&self) -> String {
        self.get_info().build.output_directory.to_string()
    }
}

// I tried to use Deserialize however I couldnt think of a good way to implement
// Deserialize trait for Program to associated Config. If there is a way I think that would
// be preferred. This trait still requires config struct implement Deserialize and we forward
// to various serde implementations that support more strait forward deserialization formats
// and provide a custom implementation for cases were we need to get data from JS modules
pub trait ConfigurationFileDeserialization: for<'a> Deserialize<'a> {

    fn from_json(s: &str) -> DoctaviousResult<Self> {
        Ok(serde_json::from_str(s)?)
    }

    fn from_yaml(s: &str) -> DoctaviousResult<Self> {
        Ok(serde_yaml::from_str(s)?)
    }

    fn from_toml(s: &str) -> DoctaviousResult<Self> {
        Ok(toml::from_str(s)?)
    }

    fn from_js_module(program: &Program) -> DoctaviousResult<Self> {
        // TODO: not implemented
        Err(DoctaviousError::Msg("not implemented".to_string()))
    }
}

pub(crate) fn read_config_files<T>(files: &Vec<&'static str>) -> DoctaviousResult<T>
    where T: ConfigurationFileDeserialization
{
    for file in files {
        let path = Path::new(&file);
        if let Some(extension) = path.extension() {
            if let Ok(content) = fs::read_to_string(&file) {
                if extension == "json" {
                    return T::from_json(content.as_str());
                } else if extension == "yaml" || extension == "yml" {
                    return T::from_yaml(content.as_str());
                } else if extension == "toml" {
                    return T::from_toml(content.as_str());
                } else if extension == "js" || extension == "ts" || extension == "mjs" || extension == "cjs" {
                    let program = parse_js_module(path.to_owned().into(), content)?;
                    return T::from_js_module(&program);
                }
            }
        }
    }

    return Err(DoctaviousError::Msg("".to_string()));
}

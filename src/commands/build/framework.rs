use std::fs;
use std::path::Path;
use std::sync::Arc;

use serde::{Deserialize};
use swc::{self, config::Options, HandlerOpts, try_with_handler};
use swc_common::{errors::{ColorConfig, Handler}, FileName, GLOBALS, SourceMap};
use swc_ecma_ast::{*, EsVersion};
use swc_ecma_parser::{EsConfig, Syntax};
use crate::commands::build::infer::Framework;

use crate::commands::build::js_module::parse_js_module;
use crate::doctavious_error::DoctaviousError;
use crate::DoctaviousResult;

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
    /// The file contains descriptive and functional metadata about a project
    /// specifically dependencies
    ///
    /// # Examples
    /// package.json, .csproj
    pub project_file: Option<&'static str>,

    // /// Detectors used to find out the framework
    // pub detection: FrameworkDetector,

    pub build: FrameworkBuildSettings,

}

pub struct FrameworkBuildSettings {
    pub command: &'static str,
    pub command_args: Option<FrameworkBuildArgs>,
    pub output_directory: &'static str
}

pub struct FrameworkBuildArgs {
    pub source: Option<FrameworkBuildArg>,
    pub config: Option<FrameworkBuildArg>,
    pub output: Option<FrameworkBuildArg>
}

pub enum FrameworkBuildArg {
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

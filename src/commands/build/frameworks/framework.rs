use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use crate::DoctaviousResult;
use serde::{Serialize, Deserialize, de};
use crate::doctavious_error::DoctaviousError;

use swc_ecma_ast::{EsVersion, *};
use swc_common::{errors::{ColorConfig, Handler}, SourceMap, GLOBALS, FileName};
use swc::{self, config::Options, try_with_handler, HandlerOpts};
use swc_ecma_parser::{EsConfig, Syntax};


pub struct FrameworkInfo {
    // id: String,

    /// Name of the framework
    ///
    /// # Examples
    /// Next.js
    pub name: String,

    /// A URL to the official website of the framework
    ///
    /// # Examples
    /// https://nextjs.org
    pub website: Option<String>,

    // Short description of the framework
    // pub description: String,

    // TODO: might not need this
    // /// The environment variable prefix
    // ///
    // /// # Examples
    // /// NEXT_PUBLIC_
    // pub envPrefix: Option<String>,

    // TODO: could be a glob?
    /// List of potential config files
    pub configs: Option<Vec<String>>,

    /// The file contains descriptive and functional metadata about a project
    /// specifically dependencies
    ///
    /// # Examples
    /// package.json, .csproj
    pub project_file: Option<String>,

    // /// Detectors used to find out the framework
    // pub detection: FrameworkDetector,
    //
    // pub build: FrameworkBuildSettings,

    // pub install_command: Box<dyn Fn(&Self) -> String>,
    // pub output_dir_name: Box<dyn Fn(&Self) -> String>,
    // pub build_command: Box<dyn Fn(&Self) -> String>
}

pub trait FrameworkSupport {

    fn get_info(&self) -> &FrameworkInfo;

    fn get_output_dir(&self) -> String {
        // default implementation...this might be necessary as we'll likely have custom for each
        // self.get_info().build.output_dir
        String::default()
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
















// TODO: parse to generic or just parse to struct with output directory
// could need to pass in key
// TODO: swap if/else with enum match or some form of iteration? would be nice
// not have to remember to add here and have compiler cause error or pick it up automatically
// fn read_config_files<'a, T: ?Sized>(files: &Vec<String>) -> DoctaviousResult<T>
// where for<'de> T: Deserialize<'de> + 'a
// {
//     for file in files {
//         let path = Path::new(&file);
//         if let Some(extension) = path.extension() {
//             if let Ok(content) = fs::read_to_string(&file) {
//                 if extension == "json" {
//                     return Ok(serde_json::from_str::<T>(content.as_str())?);
//                 } else if extension == "yaml" || extension == "yml" {
//                     return Ok(serde_yaml::from_str::<T>(content.as_str())?);
//                 } else if extension == "toml" {
//                     return Ok(toml::from_str::<T>(content.as_str())?);
//                 } else if extension == "js" || extension == "ts" || extension == "mjs" || extension == "cjs" {
//                     let program = parse_js_module(path.to_owned().into(), content)?;
//                     return from_program(program);
//                     // return Ok(program.try_into()?);
//                     // return Err(DoctaviousError::Msg("".to_string()));
//
//                 }
//             }
//         }
//     }
//
//     return Err(DoctaviousError::Msg("".to_string()));
// }

pub(crate) fn read_config_files<T>(files: &Vec<String>) -> DoctaviousResult<T>
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
                    // return Ok(program.try_into()?);
                    // return Err(DoctaviousError::Msg("".to_string()));

                }
            }
        }
    }

    return Err(DoctaviousError::Msg("".to_string()));
}

pub fn parse_js_module(filename: FileName, src: String) -> DoctaviousResult<Program> {
    let cm = Arc::<SourceMap>::default();
    let c = swc::Compiler::new(cm.clone());
    let output = GLOBALS
        .set(&Default::default(), || {
            try_with_handler(
                cm.clone(),
                HandlerOpts {
                    ..Default::default()
                },
                |handler| {
                    // println!("{}", file);
                    let fm = cm.new_source_file(filename, src);
                        //.load_file(Path::new(file))
                        //.expect("failed to load file");

                    // Ok(c.process_js_file(
                    //     fm,
                    //     handler,
                    //     &Options {
                    //         ..Default::default()
                    //     },
                    // )
                    //     .expect("failed to process file"))

                    let result = c.parse_js(
                        fm,
                        handler,
                        EsVersion::Es2020,
                        Syntax::Es(EsConfig::default()),
                        swc::config::IsModule::Bool(true),
                        None,
                    );
                    result
                },
            )
        });

    // println!("{}", serde_json::to_string(&output).unwrap());

    // TODO: wasnt sure how to use ? above as its an anyhow error and attempting from wasnt working
    match output {
        Ok(o) => Ok(o),
        Err(e) => {
            Err(DoctaviousError::Msg("failed to parse js".to_string()))
        }
    }
}



// #[cfg(test)]
// mod tests {
//     use crate::commands::build::frameworks::{Antora, Astro, DocFx, FrameworkInfo, FrameworkSupport};
//
//     #[test]
//     fn test_antora() {
//         let antora = Antora {
//             info: FrameworkInfo {
//                 name: "".to_string(),
//                 website: None,
//                 configs: Some(vec![String::from("antora-playbook.yaml")]),
//                 project_file: None,
//             },
//         };
//
//         let output = antora.get_output_dir();
//         println!("{}", output);
//     }
//
//     #[test]
//     fn test_astro() {
//         let astro = Astro {
//             info: FrameworkInfo {
//                 name: "".to_string(),
//                 website: None,
//                 configs: Some(vec![String::from("tests/resources/framework_configs/astro/astro.config.mjs")]),
//                 project_file: None,
//             },
//         };
//
//         let output = astro.get_output_dir();
//         println!("{}", output);
//     }
//
//     #[test]
//     fn test_docfx() {
//         let docfx = DocFx {
//             info: FrameworkInfo {
//                 name: "".to_string(),
//                 website: None,
//                 configs: Some(vec![String::from("tests/resources/framework_configs/docfx/docfx.json")]),
//                 project_file: None,
//             }
//         };
//
//         let output = docfx.get_output_dir();
//         println!("{}", output);
//     }
//
// }

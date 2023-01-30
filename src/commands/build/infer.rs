use std::fs;
use std::path::{Path, PathBuf};
use std::string::ToString;
use std::sync::Arc;
use lazy_static::lazy_static;
use crate::doctavious_error::Result;
use crate::settings::BuildSettings;

// use swc::config::EsVersion;
// use swc_ecma_ast::EsVersion;
// use swc_ecma_parser::EsVersion;
// use swc_ecma_ast::*;
use swc_ecma_ast::{EsVersion, *};
use swc_common::{
    errors::{ColorConfig, Handler},
    SourceMap,
    GLOBALS
};
use swc::{self, config::Options, try_with_handler, HandlerOpts};
use swc_ecma_parser::{EsConfig, Syntax};


// TODO: should we go the struct route?
// https://rust-unofficial.github.io/patterns/patterns/behavioural/strategy.html

// TODO: one issue is that often times configuration has URL which is used for site maps, etc
// not sure how vercel handles especially when it comes to preview
// https://github.com/vercel/vercel/tree/main/examples/docusaurus-2

// aka zero config

// https://github.com/netlify/build/tree/main/packages/framework-info

// If we add in functionality to try and use "build" script command how would we determine
// if that puts the output somewhere else? Ex: docusaurus build has "--out-dir"
// depending on how we build frameworks (rust structs vs json) we could include
// necessary logic when determining output dir or we encode it in json which could have a
// build setting with command and then a optional output param (maybe even --config	param)
// could need to support things like DocFX (docfx build docfx.json -o:<output_path>)
// specifying a runtime might help to remove the need to specify some options in the build settings


// FrameworkConfiguration
// files: Option<Vec<String>>
// output_directory_key: Option<String>
// to parse JS
// https://www.christopherbiscardi.com/how-to-print-a-javascript-ast-using-swc-and-rust
// We can use swc to parse the source file into an AST using parse_js.


pub trait BuildSettingOverrides {
    //
    // output
}

// Build script (config: Option<String>, output_directory: Option<String>)
// Env (output_directory_key: String) -- given we'll run it in our env this seems unnecessary


// this would be necessary for install command but for now
// lets keep it simple without it and add if/when needed - statiq might require this
// do we need a runtime so we can auto detect package manager?
// what to do with bazel?
// node - npm, yarn, pnmp
// python - pip, poetry, pdm
// ruby - gem
// rust - cargo
// .net - dotnet
// java - mvn, gradle
// kotlin - mvn, gradle
// go - go get,
// php - composer

pub struct PackageManager {
    pub id: String,
    pub name: String,
    pub command: String,
    pub detection: PackageManagerDetection
}

pub struct PackageManagerDetection {
    config_files: Vec<String>
}


// TODO: parse to generic or just parse to struct with output directory
// could need to pass in key
// pub fn read_config_files(files: &Vec<String>) -> Option<String> {
//     for file in files {
//         if let Some(extension) = Path::new(&file).extension() {
//             if let Ok(content) = fs::read_to_string(&file) {
//                 if extension == ".json" {
//                     // serde_json::from_str(content.as_str())
//                 } else if extension == ".yaml" || extension == ".yml" {
//                     // serde_yaml::from_str(content.as_str())
//                 } else if extension == ".toml" {
//                     // toml::from_str(content.as_str())
//                 } else if extension == ".js" || extension == ".mjs" {
//                     // let cm = Arc::<SourceMap>::default();
//                     // let handler = Arc::new(Handler::with_tty_emitter(
//                     //     ColorConfig::Auto,
//                     //     true,
//                     //     false,
//                     //     Some(cm.clone()),
//                     // ));
//                     // let c = swc::Compiler::new(cm.clone(), handler.clone());
//                     // let fm = cm
//                     //     .load_file(Path::new("foo.js"))
//                     //     .expect("failed to load file");
//                     // let result = c.parse_js(
//                     //     fm,
//                     //     JscTarget::Es2020,
//                     //     Syntax::Es(EsConfig::default()),
//                     //     true,
//                     //     false,
//                     // );
//
//                     let cm = Arc::<SourceMap>::default();
//                     let c = swc::Compiler::new(cm.clone());
//                     let output = GLOBALS
//                         .set(&Default::default(), || {
//                             try_with_handler(
//                                 cm.clone(),
//                                 HandlerOpts {
//                                     ..Default::default()
//                                 },
//                                 |handler| {
//                                     let fm = cm
//                                         .load_file(Path::new(content.as_str()))
//                                         .expect("failed to load file");
//
//                                     // Ok(c.process_js_file(
//                                     //     fm,
//                                     //     handler,
//                                     //     &Options {
//                                     //         ..Default::default()
//                                     //     },
//                                     // )
//                                     //     .expect("failed to process file"))
//                                     let result = c.parse_js(
//                                         fm,
//                                         handler,
//                                         EsVersion::Es2020,
//                                         Syntax::Es(EsConfig::default()),
//                                         swc::config::IsModule::Bool(true),
//                                         None,
//                                     );
//                                     result
//                                 },
//                             )
//                         })
//                         .unwrap();
//                 }
//             }
//         }
//     }
//
//     None
// }

// pub fn parse_js_module(file: &str) {
//     let cm = Arc::<SourceMap>::default();
//     let c = swc::Compiler::new(cm.clone());
//     let output = GLOBALS
//         .set(&Default::default(), || {
//             try_with_handler(
//                 cm.clone(),
//                 HandlerOpts {
//                     ..Default::default()
//                 },
//                 |handler| {
//                     println!("{}", file);
//                     let fm = cm
//                         .load_file(Path::new(file))
//                         .expect("failed to load file");
//
//                     // Ok(c.process_js_file(
//                     //     fm,
//                     //     handler,
//                     //     &Options {
//                     //         ..Default::default()
//                     //     },
//                     // )
//                     //     .expect("failed to process file"))
//                     let result = c.parse_js(
//                         fm,
//                         handler,
//                         EsVersion::Es2020,
//                         Syntax::Es(EsConfig::default()),
//                         swc::config::IsModule::Bool(true),
//                         None,
//                     );
//                     result
//                 },
//             )
//         })
//         .unwrap();
//
//     println!("{:?}", output);
// }


pub struct Framework {
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

    /// Detectors used to find out the framework
    pub detection: FrameworkDetector,

    pub build: FrameworkBuildSettings,

    // pub install_command: Box<dyn Fn(&Self) -> String>,
    // pub output_dir_name: Box<dyn Fn(&Self) -> String>,
    // pub build_command: Box<dyn Fn(&Self) -> String>
}

// TODO: handle overrides via methods or encode in json
// if json would probably need something like location (enum? config, env) and key
pub struct FrameworkBuildSettings {
    // default_install_command: String,

    pub default_build_command: String,

    // pub build_command_output_arg: Option<String>,

    pub build_command: Box<dyn Fn(&Self) -> String>,

    pub default_output_directory: String,

    pub output_directory: Box<dyn Fn(&Self) -> String>,
}

// i suppose this could include overrides
pub trait FrameworkSettingDetection {

    /// Function that returns the name of the directory that the framework outputs
    /// its build results to.
    ///
    /// This can read from configuration files or environment for example
    fn output_dir_name(&self) -> String;

    fn install_command(&self) -> String;

    fn build_command(&self) -> String;

}

pub struct FrameworkDetector {
    pub matching_strategy: FrameworkMatchingStrategy,
    pub detectors: Vec<FrameworkDetectionItem>
}

pub enum FrameworkDetectionItem {
    // TODO: should path support glob or should we force individual items?

    /// content - regex
    Config { path: String, content: Option<String> },

    /// content - regex
    Package { path: String, content: String }
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



// TODO: name
pub enum FrameworkMatchingStrategy {
    /// Strategy that requires all detectors must match
    Every, // all

    /// Strategy where one match causes the framework to be detected
    Any
}

// TODO: name? Framework Defaults?
pub struct FrameworkSettings {
    /// Default Install Command
    installCommand: String,
    /// Default Development Command
    buildCommand: String,
    /// Default Output Directory
    outputDirectory: String
}

pub enum InferrableBuildFrameworks {
    Antora,
    Astro,
    DocFX,
    Docsify,
    Docusaurus, // 1 and 2
    Eleventy,
    Gatsby,
    Hugo,
    Jekyll,
    MDBook,
    MkDocs,
    NextJS,
    NuxtJS,
    RedwoodJS,
    // Remix
    Sphinx,
    Statiq,
    VitePress,
    VuePress
}

// TODO: not sure what to call this trait
pub trait InferredFrameworkSupport {

    fn supports(self, dir: PathBuf) -> bool;

    fn build(self, dir: PathBuf) -> Result<()>;
}

impl InferredFrameworkSupport for InferrableBuildFrameworks {
    fn supports(self, dir: PathBuf) -> bool {
        match self {
            InferrableBuildFrameworks::Antora => {
                // antora-playbook.yml
            }
            InferrableBuildFrameworks::Astro => {
                // astro.config.mjs
                //
            }
            InferrableBuildFrameworks::DocFX => {
                // docfx.json
            }
            InferrableBuildFrameworks::Docsify => {
                // package.json
                // docsify
            }
            InferrableBuildFrameworks::Docusaurus => {
                // docusaurus.config.js
            }
            InferrableBuildFrameworks::Eleventy => {
                // has a zero config option so should see how to do this without the config
                // .eleventy.js

            }
            InferrableBuildFrameworks::Gatsby => {
                // gatsby-config.ts // gatsby-config.js
            }
            InferrableBuildFrameworks::Hugo => {
                // config.toml/yaml/json
                // multiple can be used
                // also has a config directory
                // has options that would need to be merged. how to handle?
                // hugo command
            }
            InferrableBuildFrameworks::Jekyll => {
                // _config.yml or _config.toml
            }
            InferrableBuildFrameworks::MDBook => {
                // book.toml
            }
            InferrableBuildFrameworks::MkDocs => {
                // mkdocs.yml
            }
            InferrableBuildFrameworks::NextJS => {
                // next.config.js / next.config.mjs
                // this is a regular Node.js module
                // could also look at package.json -> scripts -> "build": "next build",
            }
            InferrableBuildFrameworks::NuxtJS => {
                // nuxt.config.js
                // could also look at package.json -> scripts -> "build": "nuxt build",
            }
            InferrableBuildFrameworks::RedwoodJS => {
                // redwood.toml
            }
            InferrableBuildFrameworks::Sphinx => {
                // conf.py
            }
            InferrableBuildFrameworks::Statiq => {
                // https://www.statiq.dev/guide/configuration/settings
                // maybe check that its a dotnet project with Statiq.Docs package
                // look for .csproj with PackageReference of Statiq.Docs / Statiq.Web
                // or Program.cs has Statiq.Docs or Bootstrapper
                // dotnet run
            }
            InferrableBuildFrameworks::VitePress => {
                // .vitepress/config.js
                // which should export a JavaScript object:
            }
            InferrableBuildFrameworks::VuePress => {
                // .vuepress/config.js
                // which should export a JavaScript object:
                // You can also use YAML (.vuepress/config.yml) or TOML (.vuepress/config.toml) formats for the configuration file.
                // package.json -> "docs:build": "vuepress build docs"
            }
        }

        false
    }

    fn build(self, dir: PathBuf) -> Result<()> {
        match self {
            InferrableBuildFrameworks::Antora => {
                // antora antora-playbook.yml or npx antora antora-playbook.yml
                // build/site
                // change change via dir
            }
            InferrableBuildFrameworks::Astro => {
                // "npm run build"
                // astro build
                // outDir: './my-custom-build-directory'
                // defaults to "./dist"
            }
            InferrableBuildFrameworks::DocFX => {
                // "docfx <docfx_project>/docfx.json"
                // _site
            }
            InferrableBuildFrameworks::Docsify => {
                // just needs a index.html page and parses markdown
                // how to determine where docs live?
            }
            InferrableBuildFrameworks::Docusaurus => {
                // npm run build / docusaurus build
                // build directory
                // Both build/serve commands take an output dir option, and there's even a --build option on the serve command. We don't plan to add output dir to the config sorry
            }
            InferrableBuildFrameworks::Eleventy => {
                // dir.output
                // defaults to _site
            }
            InferrableBuildFrameworks::Gatsby => {
                // /public
                // people can use gatsby-plugin-output to change output dir
            }
            InferrableBuildFrameworks::Hugo => {
                // /public
                // can be changed via publishDir
            }
            InferrableBuildFrameworks::Jekyll => {
                // _site/
                // change be changed via destination
            }
            InferrableBuildFrameworks::MDBook => {
                // ./book
                // change be changed via build.build-dir
            }
            InferrableBuildFrameworks::MkDocs => {
                // site
                // change be changed via site_dir
            }
            InferrableBuildFrameworks::NextJS => {
                // .next
                // change be changed via distDir
            }
            InferrableBuildFrameworks::NuxtJS => {
                // .nuxt
                // change be changed via buildDir
            }
            InferrableBuildFrameworks::RedwoodJS => {
                // yarn rw deploy
                // matchPackage: '@redwoodjs/core',
                // ./web/dist
            }
            InferrableBuildFrameworks::Sphinx => {
                // sphinx package
                // i dont see a way to configure this outside env var
                // we could just default it ourselves
                // BUILDDIR env var
            }
            InferrableBuildFrameworks::Statiq => {
                // output
            }
            InferrableBuildFrameworks::VitePress => {
                // .vitepress/dist
                // can be configured via the outDir field
                // "docs:build": "vitepress build docs",

            }
            InferrableBuildFrameworks::VuePress => {
                // .vuepress/dist
                // can be configured via the dest field
            }
        }

        Ok(())
    }
}


// Antora,
// Astro,
// DocFX,
// Docsify,
// Docusaurus, // 1 and 2
// Eleventy,
// Gatsby,
// Hugo,
// Jekyll,
// MDBook,
// MkDocs,
// NextJS,
// NuxtJS,
// RedwoodJS,
// // Remix
// Sphinx,
// Statiq,
// VitePress,
// VuePress

// calls in constants are limited to constant functions, tuple structs and tuple variants
// lazy_static! {
//     // TODO: doctavious config will live in project directory
//     // do we also want a default settings file
//     pub static ref frameworks: Vec<Framework> = vec![
//         Framework {
//             name: "Antora".to_string(),
//             website: Some(String::from("https://antora.org/")),
//             configs: Some(vec![String::from("antora-playbook.yml")]),
//             project_file: None,
//             detection: FrameworkDetector {
//                 matching_strategy: FrameworkMatchingStrategy::Any,
//                 detectors: vec![FrameworkDetectionItem::Package { path: "".to_string(), content: "".to_string() }]
//             },
//
//             build: FrameworkBuildSettings {
//                 default_build_command: "antora".to_string(),
//                 build_command: Box::new(|_| String::default()),
//                 default_output_directory: "build/site".to_string(),
//                 output_directory: Box::new(|_| "build/site".to_string()),
//             },
//         },
//         Framework {
//             name: "Astro".to_string(),
//             website: Some(String::from("https://astro.build")),
//             configs: Some(vec![String::from("astro.config.mjs")]),
//             project_file: Some("package.json".to_string()),
//             detection: FrameworkDetector {
//                 matching_strategy: FrameworkMatchingStrategy::Every,
//                 detectors: vec![
//                     FrameworkDetectionItem::Package {
//                         path: "".to_string(),
//                         content: "astro".to_string(),
//                     }
//                 ]
//             },
//             build: FrameworkBuildSettings {
//                 default_build_command: "astro".to_string(),
//                 build_command: Box::new(|_| String::default()),
//                 default_output_directory: "dist".to_string(),
//                 output_directory: Box::new(|_| "dist".to_string()),
//             },
//         },
//         Framework {
//             name: "DocFX".to_string(),
//             website: Some(String::from("https://dotnet.github.io/docfx/")),
//             configs: Some(vec![String::from("docfx.json")]),
//             project_file: Some(".csproj".to_string()),
//             detection: FrameworkDetector {
//                 matching_strategy: FrameworkMatchingStrategy::Every,
//                 detectors: vec![
//                     FrameworkDetectionItem::Package {
//                         path: "".to_string(),
//                         content: "".to_string(),
//                     }
//                 ]
//             },
//             build: FrameworkBuildSettings {
//                 default_build_command: "docfx".to_string(),
//                 build_command: Box::new(|_| String::default()),
//                 default_output_directory: "_site".to_string(),
//                 output_directory: Box::new(|_| "_site".to_string()),
//             },
//         },
//         Framework {
//             name: "DocFX".to_string(),
//             website: Some(String::from("https://dotnet.github.io/docfx/")),
//             configs: Some(vec![String::from("docfx.json")]),
//             project_file: Some(".csproj".to_string()),
//             detection: FrameworkDetector {
//                 matching_strategy: FrameworkMatchingStrategy::Every,
//                 detectors: vec![
//                     FrameworkDetectionItem::Package {
//                         path: "".to_string(),
//                         content: "".to_string(),
//                     }
//                 ]
//             },
//             build: FrameworkBuildSettings {
//                 default_build_command: "docfx".to_string(),
//                 build_command: Box::new(|_| String::default()),
//                 default_output_directory: "_site".to_string(),
//                 output_directory: Box::new(|_| "_site".to_string()),
//             },
//         },
//         // TODO: how to detect? perhaps we dont support this
//         Framework {
//             name: "Docsify".to_string(),
//             website: Some(String::from("https://docsify.js.org/#/")),
//             configs: None,
//             project_file: Some(String::from("package.json")),
//             detection: FrameworkDetector {
//                 matching_strategy: FrameworkMatchingStrategy::Every,
//                 detectors: vec![
//                     FrameworkDetectionItem::Package {
//                         path: "".to_string(),
//                         content: "".to_string(),
//                     }
//                 ]
//             },
//             build: FrameworkBuildSettings {
//                 default_build_command: "".to_string(),
//                 build_command: Box::new(|_| String::default()),
//                 default_output_directory: "docs".to_string(),
//                 output_directory: Box::new(|_| "docs".to_string()),
//             },
//         },
//
//         // siteConfig.js"
//         Framework {
//             name: "Docusarus 1".to_string(),
//             website: Some(String::from("https://v1.docusaurus.io/")),
//             configs: Some(vec![String::from("siteConfig.js")]),
//             project_file: Some(String::from("package.json")),
//             detection: FrameworkDetector {
//                 matching_strategy: FrameworkMatchingStrategy::Every,
//                 detectors: vec![
//                     FrameworkDetectionItem::Package {
//                         path: "".to_string(),
//                         content: "docusaurus".to_string(),
//                     }
//                 ]
//             },
//             build: FrameworkBuildSettings {
//                 default_build_command: "docusaurus-build".to_string(),
//                 build_command: Box::new(|_| String::default()),
//                 default_output_directory: "build".to_string(),
//                 output_directory: Box::new(|_| "build".to_string()),
//             },
//         },
//         Framework {
//             name: "Docusaurus 2".to_string(),
//             website: Some(String::from("https://docusaurus.io/")),
//             configs: Some(vec![String::from("docusaurus.config.js")]),
//             project_file: Some(String::from("package.json")),
//             detection: FrameworkDetector {
//                 matching_strategy: FrameworkMatchingStrategy::Every,
//                 detectors: vec![
//                     FrameworkDetectionItem::Package {
//                         path: "".to_string(),
//                         content: "@docusaurus/core".to_string(),
//                     }
//                 ]
//             },
//             build: FrameworkBuildSettings {
//                 default_build_command: "docusaurus".to_string(),
//                 build_command: Box::new(|_| String::default()),
//                 default_output_directory: "build".to_string(),
//                 output_directory: Box::new(|_| "build".to_string()),
//             },
//         },
//         Framework {
//             name: "Jekyll".to_string(),
//             website: Some(String::from("https://jekyllrb.com/")),
//             configs: Some(vec![String::from("_config.yml"), String::from("_config.toml")]),
//             // TODO: should this look at Gemfile.lock as well?
//             project_file: Some(String::from("Gemfile")),
//             detection: FrameworkDetector {
//                 matching_strategy: FrameworkMatchingStrategy::Any,
//                 detectors: vec![
//                     FrameworkDetectionItem::Package {
//                         path: "".to_string(),
//                         content: "jekyll".to_string(),
//                     },
//                     FrameworkDetectionItem::Package {
//                         path: "".to_string(),
//                         content: "jekyll_plugins".to_string(),
//                     }
//                 ]
//             },
//             build: FrameworkBuildSettings {
//                 default_build_command: "jekyll build".to_string(),
//                 build_command: Box::new(|_| String::default()),
//                 default_output_directory: "_site".to_string(),
//                 // config destination override
//                 output_directory: Box::new(|_| "_site".to_string()),
//             },
//         },
//         Framework {
//             name: "Solid Start".to_string(),
//             website: Some(String::from("https://start.solidjs.com/")),
//             configs: Some(vec![String::from("vite.config.ts")]),
//             project_file: Some("package.json".to_string()),
//             detection: FrameworkDetector {
//                 matching_strategy: FrameworkMatchingStrategy::Every,
//                 detectors: vec![
//                     FrameworkDetectionItem::Package {
//                         path: "".to_string(),
//                         content: "solid-js".to_string(),
//                     },
//                     // TODO: this one is probably all we need
//                     FrameworkDetectionItem::Package {
//                         path: "".to_string(),
//                         content: "solid-start".to_string(),
//                     }
//                 ]
//             },
//             build: FrameworkBuildSettings {
//                 default_build_command: "solid-start build".to_string(),
//                 build_command: Box::new(|_| String::default()),
//                 default_output_directory: "dist".to_string(), // .output?
//                 output_directory: Box::new(|_| "dist".to_string()),
//             },
//         }
//     ];
// }



// supports return boolean
// build -> void or maybe dir so that it can be passed to deploy or have a method to infer/determine build directory output

#[cfg(test)]
mod tests {
    // use crate::commands::build::infer::parse_js_module;
    //
    // #[test]
    // fn test_parse_js_v1() {
    //
    //     parse_js_module("next_js_v1.mjs")
    // }
    //
    // #[test]
    // fn test_parse_js_v2() {
    //
    //     parse_js_module("next_js_v2.mjs")
    // }

}

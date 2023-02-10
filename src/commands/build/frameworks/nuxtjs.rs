// nuxt.config.js
// could also look at package.json -> scripts -> "build": "nuxt build",

// .nuxt --> default
// change be changed via buildDir

// nuxt v2 for static pre-rendered
// nuxt generate
// dist/

use serde::{Deserialize};
use swc_ecma_ast::{Lit, Program};
use swc_ecma_ast::ModuleDecl::ExportDefaultExpr;

use crate::commands::build::framework::{
    ConfigurationFileDeserialization,
    FrameworkBuildArg,
    FrameworkBuildArgs,
    FrameworkBuildSettings,
    FrameworkInfo,
    FrameworkSupport,
    read_config_files
};
use crate::commands::build::js_module::{get_string_property_value, PropertyAccessor};
use crate::doctavious_error::Result as DoctaviousResult;
use crate::doctavious_error::DoctaviousError;

#[derive(Deserialize)]
struct NuxtJSConfig { output: Option<String> }

pub struct NuxtJS { info: FrameworkInfo }

impl NuxtJS {
    fn new(configs: Option<Vec<&'static str>>) -> Self {
        Self {
            info: FrameworkInfo {
                name: "Nuxt",
                website: Some("https://nuxtjs.org/"),
                configs,
                project_file: None,
                build: FrameworkBuildSettings {
                    command: "nuxt build",
                    command_args: None,
                    output_directory: ".nuxt",
                },
            }
        }
    }
}

impl Default for NuxtJS {
    fn default() -> Self {
        NuxtJS::new(Some(Vec::from(["nuxt.config.js"])))
    }
}


impl FrameworkSupport for NuxtJS {
    fn get_info(&self) -> &FrameworkInfo {
        &self.info
    }

    fn get_output_dir(&self) -> String {
        if let Some(configs) = &self.info.configs {
            match read_config_files::<NuxtJSConfig>(configs) {
                Ok(c) => {
                    if let Some(dest) = c.output {
                        return dest;
                    }
                }
                Err(e) => {
                    // log warning/error
                    println!("{}", e.to_string());
                }
            }
        }

        self.info.build.output_directory.to_string()
    }
}

impl ConfigurationFileDeserialization for NuxtJSConfig {

    fn from_js_module(program: &Program) -> DoctaviousResult<Self> {
        if let Some(module) = program.as_module() {
            let output = module.get_property_as_string("buildDir");
            if output.is_some() {
                return Ok(Self {
                    output
                });
            }
            // for item in &module.body {
            //     if let Some(ExportDefaultExpr(export_expression)) = item.as_module_decl() {
            //         if let Some(obj) = export_expression.expr.as_object() {
            //             let output = get_string_property_value(&obj.props, "buildDir");
            //             if output.is_some() {
            //                 return Ok(Self {
            //                     output
            //                 });
            //             }
            //         }
            //     }
            // }
        }
        Err(DoctaviousError::Msg("invalid config".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::build::framework::{FrameworkBuildSettings, FrameworkInfo, FrameworkSupport};
    use super::NuxtJS;

    #[test]
    fn test_nuxtjs() {
        for config in ["tests/resources/framework_configs/nuxtjs/nuxt.config.js"] {
            let nuxtjs = NuxtJS::new(Some(vec![config]));

            let output = nuxtjs.get_output_dir();
            assert_eq!(output, String::from("build"))
        }

    }

}

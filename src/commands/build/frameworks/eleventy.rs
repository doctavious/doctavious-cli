// .eleventy.js
//
// .eleventy.js
// eleventy.config.js Added in v2.0.0-beta.1
// eleventy.config.cjs Added in v2.0.0-beta.1

// dir.output
// defaults to _site


use serde::{Deserialize};
use swc_ecma_ast::{Lit, Program};
use swc_ecma_ast::Stmt::Expr;
use crate::commands::build::framework::{
    ConfigurationFileDeserialization,
    FrameworkBuildArg,
    FrameworkBuildArgs,
    FrameworkBuildSettings,
    FrameworkInfo,
    FrameworkSupport,
    read_config_files
};
use crate::commands::build::js_module::{get_assignment_function, get_function_return_obj, get_obj_property, get_string_property_value};
use crate::doctavious_error::DoctaviousError;
use crate::doctavious_error::{Result as DoctaviousResult};

#[derive(Deserialize)]
struct EleventyConfig { output: String }

pub struct Eleventy { info: FrameworkInfo }

impl Default for Eleventy {
    fn default() -> Self {
        Self {
            info: FrameworkInfo {
                name: "Eleventy",
                website: Some("https://www.11ty.dev/"),
                configs: Some(Vec::from([".eleventy.js", "eleventy.config.js", "eleventy.config.cjs"])),
                project_file: None,
                build: FrameworkBuildSettings {
                    command: "eleventy",
                    command_args: Some(FrameworkBuildArgs {
                        source: None,
                        config: None,
                        output: Some(FrameworkBuildArg::Option {
                            short: "",
                            long: "--output"
                        })
                    }),
                    output_directory: "_site",
                },
            }
        }
    }
}


impl FrameworkSupport for Eleventy {
    fn get_info(&self) -> &FrameworkInfo {
        &self.info
    }

    fn get_output_dir(&self) -> String {
        if let Some(configs) = &self.info.configs {
            match read_config_files::<EleventyConfig>(configs) {
                Ok(c) => {
                    return c.output;
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

impl ConfigurationFileDeserialization for EleventyConfig {

    fn from_js_module(program: &Program) -> DoctaviousResult<Self> {
        if let Some(func) = get_assignment_function(program) {
            if let Some(return_obj) = get_function_return_obj(&func) {
                if let Some(dir_prop) = get_obj_property(&return_obj, "dir") {
                    if let Some(output) = get_string_property_value(&dir_prop.props, "output") {
                        return Ok(Self {
                            output
                        });
                    }
                }
            }
        }

        Err(DoctaviousError::Msg("invalid config".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::build::framework::{FrameworkBuildSettings, FrameworkInfo, FrameworkSupport};
    use super::Eleventy;

    #[test]
    fn test_eleventy() {
        let eleventy = Eleventy {
            info: FrameworkInfo {
                name: "",
                website: None,
                configs: Some(vec!["tests/resources/framework_configs/eleventy/.eleventy.js"]),
                project_file: None,
                build: FrameworkBuildSettings {
                    command: "",
                    command_args: None,
                    output_directory: "",
                },
            },
        };

        let output = eleventy.get_output_dir();
        assert_eq!(output, String::from("dist"))
    }

}

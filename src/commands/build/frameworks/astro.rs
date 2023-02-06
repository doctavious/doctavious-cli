// astro.config.mjs
// "npm run build"
// astro build
// outDir: './my-custom-build-directory'
// defaults to "./dist"


use serde::{Deserialize};
use swc_ecma_ast::{Lit, ModuleDecl, Program};
use crate::commands::build::frameworks::framework::{ConfigurationFileDeserialization, FrameworkBuildArg, FrameworkBuildArgs, FrameworkBuildSettings, FrameworkInfo, FrameworkSupport, read_config_files};
use crate::commands::build::js_module::{get_call_expression, get_call_string_property};
use crate::doctavious_error::DoctaviousError;
use crate::DoctaviousResult;

pub struct Astro { info: FrameworkInfo }

impl Default for Astro {
    fn default() -> Self {
        Self {
            info: FrameworkInfo {
                name: "Astro",
                website: Some("https://astro.build"),
                configs: Some(Vec::from(["astro.config.mjs"])),
                project_file: None,
                build: FrameworkBuildSettings {
                    command: "astro build",
                    command_args: Some(FrameworkBuildArgs {
                        source: None,
                        config: Some(FrameworkBuildArg::Option {
                            short: "",
                            long: "--config",
                        }),
                        output: None,
                    }),
                    output_directory: "./dist",
                },
            }
        }
    }
}

impl FrameworkSupport for Astro {
    fn get_info(&self) -> &FrameworkInfo {
        &self.info
    }

    fn get_output_dir(&self) -> String {
        if let Some(configs) = &self.info.configs {
            match read_config_files::<AstroConfig>(configs) {
                Ok(c) => {
                    return c.output
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

#[derive(Deserialize)]
struct AstroConfig { output: String }

impl ConfigurationFileDeserialization for AstroConfig {

    fn from_js_module(program: &Program) -> DoctaviousResult<Self> {
        let define_config = get_call_expression(program, "defineConfig");
        if let Some(define_config) = define_config {
            if let Some(val) = get_call_string_property(&define_config, "outDir") {
                return Ok(Self {
                    output: val
                });
            }
        }

        Err(DoctaviousError::Msg("invalid config".to_string()))
    }
}


#[cfg(test)]
mod tests {
    use crate::commands::build::frameworks::framework::{FrameworkBuildSettings, FrameworkInfo, FrameworkSupport};
    use super::Astro;

    #[test]
    fn test_astro() {
        let astro = Astro {
            info: FrameworkInfo {
                name: "",
                website: None,
                configs: Some(vec!["tests/resources/framework_configs/astro/astro.config.mjs"]),
                project_file: None,
                build: FrameworkBuildSettings {
                    command: "",
                    command_args: None,
                    output_directory: "",
                },
            },
        };

        let output = astro.get_output_dir();
        assert_eq!(output, "./build")
    }

}

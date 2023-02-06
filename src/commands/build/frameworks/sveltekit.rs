// defaults to ".svelte-kit"
// svelte.config.js
// outDir overrides
// dependency - adapter-static


use serde::{Deserialize};
use swc_ecma_ast::{Lit, Program};
use swc_ecma_ast::Stmt::{Decl};

use crate::commands::build::frameworks::framework::{ConfigurationFileDeserialization, FrameworkBuildArg, FrameworkBuildArgs, FrameworkBuildSettings, FrameworkInfo, FrameworkSupport, read_config_files};
use crate::commands::build::js_module::{get_string_property_value, get_variable_declaration, get_variable_properties};
use crate::doctavious_error::{DoctaviousError, Result as DoctaviousResult};

// TODO: given there is no option to override does it make sense to still enforce Deserialize
// and ConfigurationFileDeserialization?
// I suppose we can determine if gatsby-plugin-output is in the plugins and grab it from there
#[derive(Deserialize)]
struct SvelteKitConfig { output: Option<String> }

pub struct SvelteKit { info: FrameworkInfo }

impl Default for SvelteKit {
    fn default() -> Self {
        Self {
            info: FrameworkInfo {
                name: "SvelteKit",
                website: Some("https://kit.svelte.dev/"),
                configs: Some(vec!["svelte.config.js"]),
                project_file: None,
                build: FrameworkBuildSettings {
                    command: "vite build",
                    command_args: Some(FrameworkBuildArgs {
                        source: None,
                        config: None,
                        output: Some(FrameworkBuildArg::Option {
                            short: "",
                            long: "--outDir"
                        })
                    }),
                    // TODO: validate
                    // according to the following https://github.com/netlify/build/pull/4823
                    // .svelte-kit is the internal build dir, not the publish dir.
                    output_directory: "build" //".svelte-kit",
                },
            },
        }
    }
}

impl FrameworkSupport for SvelteKit {
    fn get_info(&self) -> &FrameworkInfo {
        &self.info
    }

    fn get_output_dir(&self) -> String {
        if let Some(configs) = &self.info.configs {
            match read_config_files::<SvelteKitConfig>(configs) {
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

impl ConfigurationFileDeserialization for SvelteKitConfig {
    fn from_js_module(program: &Program) -> DoctaviousResult<Self> {
        let var = get_variable_declaration(program, "config");
        if let Some(var) = var {
            let properties = get_variable_properties(var, "kit");
            if let Some(properties) = properties {
                let output = get_string_property_value(properties, "outDir");
                if output.is_some() {
                    return Ok(Self {
                        output
                    });
                }
            }
        }

        return Err(DoctaviousError::Msg("".to_string()));
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::build::frameworks::framework::{FrameworkBuildSettings, FrameworkInfo, FrameworkSupport};
    use super::SvelteKit;

    #[test]
    fn test_sveltekit() {
        let sveltekit = SvelteKit {
            info: FrameworkInfo {
                name: "",
                website: None,
                configs: Some(vec!["tests/resources/framework_configs/sveltekit/svelte.config.js"]),
                project_file: None,
                build: FrameworkBuildSettings {
                    command: "",
                    command_args: None,
                    output_directory: "",
                },
            }
        };

        let output = sveltekit.get_output_dir();
        assert_eq!(output, "build")
    }

}

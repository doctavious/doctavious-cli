// nuxt.config.js
// could also look at package.json -> scripts -> "build": "nuxt build",

// .nuxt --> default
// change be changed via buildDir

// nuxt v2 for static pre-rendered
// nuxt generate
// dist/

use serde::{Serialize, Deserialize, de};
use swc_ecma_ast::{Lit, ModuleDecl, ModuleItem, Program, Stmt};
use swc_ecma_ast::ModuleDecl::ExportDefaultExpr;
use swc_ecma_ast::Stmt::{Decl, Expr};
use crate::commands::build::frameworks::framework::{ConfigurationFileDeserialization, FrameworkBuildSettings, FrameworkInfo, FrameworkSupport, read_config_files};
use crate::doctavious_error::DoctaviousError;
use crate::doctavious_error::{Result as DoctaviousResult};

#[derive(Deserialize)]
struct NuxtJSConfig { output: Option<String> }

pub struct NuxtJS { info: FrameworkInfo }

impl Default for NuxtJS {
    fn default() -> Self {
        Self {
            info: FrameworkInfo {
                name: "Nuxt",
                website: Some("https://nuxtjs.org/"),
                configs: Some(Vec::from(["nuxt.config.js"])),
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

// NUXTJS: NuxtJS = NuxtJS {
//     info: FrameworkInfo {
//         name: "Nuxt",
//         website: Some("https://nuxtjs.org/"),
//         configs: Some(Vec::from(["nuxt.config.js"])),
//         project_file: None,
//     },
// };


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
        // TODO: try and simplify
        println!("{}", serde_json::to_string(program)?);
        if let Some(module) = program.as_module() {
            for item in &module.body {
                if let Some(ExportDefaultExpr(export_expression)) = item.as_module_decl() {
                    if let Some(obj) = export_expression.expr.as_object() {
                        for props in &obj.props {
                            if let Some(dir_prop) = props.as_prop() {
                                if let Some(kv) = (*dir_prop).as_key_value() {
                                    if let Some(ident) = kv.key.as_ident() {
                                        if ident.sym.as_ref() == "buildDir" {
                                            if let Some(Lit::Str(s)) = &kv.value.as_lit() {
                                                return Ok(Self {
                                                    output: Some(s.value.to_string())
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(DoctaviousError::Msg("invalid config".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::build::frameworks::framework::{FrameworkBuildSettings, FrameworkInfo, FrameworkSupport};
    use super::NuxtJS;

    #[test]
    fn test_nuxtjs() {
        for config in ["tests/resources/framework_configs/nuxtjs/nuxt.config.js"] {
            let nuxtjs = NuxtJS {
                info: FrameworkInfo {
                    name: "",
                    website: None,
                    configs: Some(vec![config]),
                    project_file: None,
                    build: FrameworkBuildSettings {
                        command: "",
                        command_args: None,
                        output_directory: "",
                    },
                },
            };

            let output = nuxtjs.get_output_dir();
            assert_eq!(output, String::from("build"))
        }

    }

}

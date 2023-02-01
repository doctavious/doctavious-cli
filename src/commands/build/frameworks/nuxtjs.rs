// nuxt.config.js
// could also look at package.json -> scripts -> "build": "nuxt build",

// .nuxt --> default
// change be changed via buildDir

use serde::{Serialize, Deserialize, de};
use swc_ecma_ast::{Lit, ModuleDecl, ModuleItem, Program, Stmt};
use swc_ecma_ast::ModuleDecl::ExportDefaultExpr;
use swc_ecma_ast::Stmt::{Decl, Expr};
use crate::commands::build::frameworks::framework::{ConfigurationFileDeserialization, FrameworkInfo, FrameworkSupport, read_config_files};
use crate::doctavious_error::DoctaviousError;
use crate::doctavious_error::{Result as DoctaviousResult};

#[derive(Deserialize)]
struct NuxtJSConfig { output: Option<String> }

pub struct NuxtJS { info: FrameworkInfo }
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

        //self.info.build.outputdir
        String::default()
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
    use crate::commands::build::frameworks::framework::{FrameworkInfo, FrameworkSupport};
    use super::NuxtJS;

    #[test]
    fn test_nuxtjs() {
        for config in ["tests/resources/framework_configs/nuxtjs/nuxt.config.js"] {
            let nuxtjs = NuxtJS {
                info: FrameworkInfo {
                    name: "".to_string(),
                    website: None,
                    configs: Some(vec![config.to_string()]),
                    project_file: None,
                },
            };

            let output = nuxtjs.get_output_dir();
            assert_eq!(output, String::from("build"))
        }

    }

}

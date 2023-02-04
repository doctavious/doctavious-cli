// .eleventy.js
//
// .eleventy.js
// eleventy.config.js Added in v2.0.0-beta.1
// eleventy.config.cjs Added in v2.0.0-beta.1

// dir.output
// defaults to _site


use serde::{Serialize, Deserialize, de};
use swc_ecma_ast::{Lit, ModuleDecl, ModuleItem, Program, Stmt};
use swc_ecma_ast::Stmt::Expr;
use crate::commands::build::frameworks::framework::{ConfigurationFileDeserialization, FrameworkBuildArg, FrameworkBuildArgs, FrameworkBuildOption, FrameworkBuildSettings, FrameworkInfo, FrameworkSupport, read_config_files};
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
                        config: None,
                        output: Some(FrameworkBuildArg::Option(FrameworkBuildOption {
                            short: "",
                            long: "--output"
                        }))
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

            // for config in configs {
            //     // TODO: this might be better to parse into struct
            //     println!("{}", config);
            //     if let Ok(contents) = fs::read_to_string(config) {
            //         match serde_yaml::from_str::<AntoraConfig>(contents.as_str()) {
            //             Ok(c) => {
            //                 return c.output.dir
            //             }
            //             Err(e) => {
            //                 // log warning/error
            //                 println!("{}", e.to_string());
            //             }
            //         }
            //     } else {
            //         println!("could not read file {}", config);
            //     }
            // }
        }

        //self.info.build.outputdir
        String::default()
    }
}

impl ConfigurationFileDeserialization for EleventyConfig {

    fn from_js_module(program: &Program) -> DoctaviousResult<Self> {
        // TODO: try and simplify
        if let Some(module) = program.as_module() {
            for item in &module.body {
                if let Some(Expr(stmt)) = item.as_stmt() {
                    let expression = &*stmt.expr;
                    if let Some(assign) = expression.as_assign() {
                        let rhs = &*assign.right;
                        if let Some(func) = rhs.as_fn_expr() {
                            let fn_expr = &*func.function;
                            if let Some(fn_body) = &fn_expr.body {
                                for statement in &fn_body.stmts {
                                    if let Some(return_statement) = statement.as_return_stmt() {
                                        if let Some(return_statement_args) = &return_statement.arg {
                                            let args = &**return_statement_args;
                                            if let Some(obj_expression) = args.as_object() {
                                                for prop in &obj_expression.props {
                                                    if let Some(p) = prop.as_prop() {
                                                        if let Some(kv) = (*p).as_key_value() {
                                                            if let Some(ident) = kv.key.as_ident() {
                                                                if ident.sym.as_ref() == "dir" {
                                                                    if let Some(dir_obj) = kv.value.as_object() {
                                                                        for dir_prop in &dir_obj.props {
                                                                            if let Some(dir_prop) = dir_prop.as_prop() {
                                                                                if let Some(kv) = (*dir_prop).as_key_value() {
                                                                                    if let Some(ident) = kv.key.as_ident() {
                                                                                        if ident.sym.as_ref() == "output" {
                                                                                            if let Some(Lit::Str(s)) = &kv.value.as_lit() {
                                                                                                return Ok(Self {
                                                                                                    output: s.value.to_string()
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
            }
        }
        Err(DoctaviousError::Msg("invalid config".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::build::frameworks::framework::{FrameworkBuildSettings, FrameworkInfo, FrameworkSupport};
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

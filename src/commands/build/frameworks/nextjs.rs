// next.config.js / next.config.mjs
// this is a regular Node.js module
// could also look at package.json -> scripts -> "build": "next build",

// .next -> default directory
// change be changed via distDir

use serde::{Serialize, Deserialize, de};
use swc_ecma_ast::{Lit, ModuleDecl, ModuleItem, Program, Stmt};
use swc_ecma_ast::Stmt::{Decl, Expr};
use crate::commands::build::frameworks::framework::{ConfigurationFileDeserialization, FrameworkInfo, FrameworkSupport, read_config_files};
use crate::doctavious_error::DoctaviousError;
use crate::doctavious_error::{Result as DoctaviousResult};

#[derive(Deserialize)]
struct NextJSConfig { output: String }

pub struct NextJS { info: FrameworkInfo }

impl Default for NextJS {
    fn default() -> Self {
        Self {
            info: FrameworkInfo {
                name: "Next.js",
                website: Some("https://nextjs.org/"),
                configs: Some(Vec::from(["next.config.js", "next.config.mjs"])),
                project_file: None,
            }
        }
    }
}

impl FrameworkSupport for NextJS {
    fn get_info(&self) -> &FrameworkInfo {
        &self.info
    }

    fn get_output_dir(&self) -> String {
        if let Some(configs) = &self.info.configs {
            match read_config_files::<NextJSConfig>(configs) {
                Ok(c) => {
                    return c.output;
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

impl ConfigurationFileDeserialization for NextJSConfig {

    fn from_js_module(program: &Program) -> DoctaviousResult<Self> {
        // TODO: try and simplify
        println!("{}", serde_json::to_string(program)?);
        if let Some(module) = program.as_module() {
            for item in &module.body {
                if let Some(Decl(decl)) = item.as_stmt() {
                    if let Some(variable_decl) = decl.as_var() {
                        let variable = &**variable_decl;
                        for declaration in &variable.decls {
                            if let Some(init_decl) = &declaration.init {
                                if let Some(init_decl_obj) = init_decl.as_object() {
                                    for props in &init_decl_obj.props {
                                        if let Some(dir_prop) = props.as_prop() {
                                            if let Some(kv) = (*dir_prop).as_key_value() {
                                                if let Some(ident) = kv.key.as_ident() {
                                                    if ident.sym.as_ref() == "distDir" {
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
                } else if let Some(Expr(stmt)) = item.as_stmt() {
                    let expression = &*stmt.expr;
                    if let Some(assign) = expression.as_assign() {
                        let rhs = &*assign.right;
                        if let Some(obj) = rhs.as_object() {
                            for prop in &obj.props {
                                if let Some(p) = prop.as_prop() {
                                    if let Some(kv) = p.as_key_value() {
                                        if let Some(ident) = kv.key.as_ident() {
                                            if ident.sym.as_ref() == "distDir" {
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
        Err(DoctaviousError::Msg("invalid config".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::build::frameworks::framework::{FrameworkInfo, FrameworkSupport};
    use super::NextJS;

    #[test]
    fn test_nextjs() {
        for config in ["tests/resources/framework_configs/nextjs/next_js_v1.mjs", "tests/resources/framework_configs/nextjs/next_js_v2.mjs"] {
            let nextjs = NextJS {
                info: FrameworkInfo {
                    name: "",
                    website: None,
                    configs: Some(vec![config]),
                    project_file: None,
                },
            };

            let output = nextjs.get_output_dir();
            assert_eq!(output, String::from("build"))
        }

    }

}

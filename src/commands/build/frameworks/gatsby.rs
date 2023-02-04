// gatsby-config.ts // gatsby-config.js

// /public
// people can use gatsby-plugin-output to change output dir

// gatsby build

use serde::{Serialize, Deserialize, de};
use swc_ecma_ast::{ArrayLit, Lit, ModuleDecl, ModuleItem, ObjectLit, Program, Stmt};
use swc_ecma_ast::Expr::{Array, Object, Tpl};
use swc_ecma_ast::Stmt::{Decl, Expr};

use crate::commands::build::frameworks::framework::{ConfigurationFileDeserialization, FrameworkBuildSettings, FrameworkInfo, FrameworkSupport, read_config_files};
use crate::doctavious_error::{DoctaviousError, Result as DoctaviousResult};

// TODO: given there is no option to override does it make sense to still enforce Deserialize
// and ConfigurationFileDeserialization?
// I suppose we can determine if gatsby-plugin-output is in the plugins and grab it from there
#[derive(Deserialize)]
struct GatsbyConfig { output: String }

pub struct Gatsby { info: FrameworkInfo }

impl Default for Gatsby {
    fn default() -> Self {
        Self {
            info: FrameworkInfo {
                name: "Gatsby",
                website: Some("https://www.gatsbyjs.com/"),
                configs: Some(Vec::from(["gatsby-config.js", "gatsby-config.ts"])),
                project_file: None,
                build: FrameworkBuildSettings {
                    command: "gatsby build",
                    command_args: None,
                    output_directory: "/public",
                },
            }
        }
    }
}

impl FrameworkSupport for Gatsby {
    fn get_info(&self) -> &FrameworkInfo {
        &self.info
    }

    fn get_output_dir(&self) -> String {
        if let Some(configs) = &self.info.configs {
            match read_config_files::<GatsbyConfig>(configs) {
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

impl ConfigurationFileDeserialization for GatsbyConfig {
    fn from_js_module(program: &Program) -> DoctaviousResult<Self> {
        // TODO: try and simplify
        println!("{}", serde_json::to_string(program)?);
        if let Some(module) = program.as_module() {
            for item in &module.body {
                if let Some(Expr(stmt)) = item.as_stmt() {
                    let expression = &*stmt.expr;
                    if let Some(assign) = expression.as_assign() {
                        let rhs = &*assign.right;
                        if let Some(obj) = rhs.as_object() {
                            for prop in &obj.props {
                                if let Some(p) = prop.as_prop() {
                                    if let Some(kv) = p.as_key_value() {
                                        if let Some(ident) = kv.key.as_ident() {
                                            if ident.sym.as_ref() == "plugins" {
                                                if let Some(arr) = kv.value.as_array() {
                                                    for elem in &arr.elems {
                                                        if let Some(exp) = elem {
                                                            if let Some(o) = exp.expr.as_object() {
                                                                for op in &o.props {
                                                                    if let Some(op2) = op.as_prop() {
                                                                        if let Some(kv) = op2.as_key_value() {
                                                                            if let Some(ident) = kv.key.as_ident() {
                                                                                if ident.sym.as_ref() == "resolve" {
                                                                                    // TODO: could this also be a literal?
                                                                                    if let Some(template) = kv.value.as_tpl() {
                                                                                        for q in &template.quasis {
                                                                                            if let Some(cooked) = &q.cooked {
                                                                                                // TODO: if dont have plugin then default
                                                                                                if cooked == "gatsby-plugin-output" {

                                                                                                }
                                                                                            }
                                                                                        }
                                                                                    }
                                                                                } else if ident.sym.as_ref() == "options" {
                                                                                    if let Some(o3) = &kv.value.as_object() {
                                                                                        for p2 in &o3.props {
                                                                                            if let Some(p) = p2.as_prop() {
                                                                                                if let Some(kv) = p.as_key_value() {
                                                                                                    if let Some(ident) = kv.key.as_ident() {
                                                                                                        if ident.sym.as_ref() == "publicPath" {
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


        return Err(DoctaviousError::Msg("".to_string()));
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::build::frameworks::framework::{FrameworkBuildSettings, FrameworkInfo, FrameworkSupport};
    use super::Gatsby;

    #[test]
    fn test_gatsby() {
        let gatsby = Gatsby {
            info: FrameworkInfo {
                name: "",
                website: None,
                configs: Some(vec!["tests/resources/framework_configs/gatsby/gatsby-config.js"]),
                project_file: None,
                build: FrameworkBuildSettings {
                    command: "",
                    command_args: None,
                    output_directory: "",
                },
            }
        };

        let output = gatsby.get_output_dir();
        assert_eq!(output, "dist")
    }

}

// .vitepress/config.js
// which should export a JavaScript object:
// .vitepress/dist
// can be configured via the outDir field
// "docs:build": "vitepress build docs",

// .js, .ts, .cjs, .mjs, .cts, .mts

use serde::{Serialize, Deserialize, de};
use swc_ecma_ast::{Lit, ModuleDecl, ModuleItem, Program, Stmt};
use swc_ecma_ast::ModuleDecl::ExportDefaultExpr;
use swc_ecma_ast::Stmt::{Decl, Expr};
use crate::commands::build::frameworks::framework::{ConfigurationFileDeserialization, FrameworkInfo, FrameworkSupport, read_config_files};
use crate::doctavious_error::DoctaviousError;
use crate::doctavious_error::{Result as DoctaviousResult};

#[derive(Deserialize)]
struct VitePressConfig { output: Option<String> }

pub struct VitePress { info: FrameworkInfo }
impl FrameworkSupport for VitePress {
    fn get_info(&self) -> &FrameworkInfo {
        &self.info
    }

    fn get_output_dir(&self) -> String {
        if let Some(configs) = &self.info.configs {
            match read_config_files::<VitePressConfig>(configs) {
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

impl ConfigurationFileDeserialization for VitePressConfig {

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
                                                    if ident.sym.as_ref() == "outDir" {
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
                } else if let Some(ExportDefaultExpr(export_expression)) = item.as_module_decl() {
                    // callee value "defineConfig"
                    if let Some(call) = export_expression.expr.as_call() {
                        for call_arg in &call.args {
                            if let Some(obj) = call_arg.expr.as_object() {
                                for props in &obj.props {
                                    if let Some(dir_prop) = props.as_prop() {
                                        if let Some(kv) = (*dir_prop).as_key_value() {
                                            if let Some(ident) = kv.key.as_ident() {
                                                if ident.sym.as_ref() == "outDir" {
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
            }
        }
        Err(DoctaviousError::Msg("invalid config".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::build::frameworks::framework::{FrameworkInfo, FrameworkSupport};
    use super::VitePress;

    #[test]
    fn test_vitepress() {
        let configs = [
            "tests/resources/framework_configs/vitepress/config.js",
            "tests/resources/framework_configs/vitepress/config.ts",
        ];
        for config in configs {
            let vitepress = VitePress {
                info: FrameworkInfo {
                    name: "".to_string(),
                    website: None,
                    configs: Some(vec![config.to_string()]),
                    project_file: None,
                },
            };

            let output = vitepress.get_output_dir();
            assert_eq!(output, String::from("build"))
        }

    }

}

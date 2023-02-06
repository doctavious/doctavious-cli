// .vuepress/config.js
// inside docs directory
// which should export a JavaScript object:
// You can also use YAML (.vuepress/config.yml) or TOML (.vuepress/config.toml) formats for the configuration file.
// package.json -> "docs:build": "vuepress build docs"

// vuepress build [targetDir] -d, --dest <dest>

// .vuepress/dist
// can be configured via the dest field

// .vuepress/config.js
// .vuepress/config.yml
// .vuepress/config.toml
// .vuepress/config.ts


use serde::{Deserialize};
use swc_ecma_ast::{Lit, Program};
use swc_ecma_ast::ModuleDecl::ExportDefaultExpr;
use swc_ecma_ast::Stmt::{Expr};
use crate::commands::build::framework::{
    ConfigurationFileDeserialization,
    FrameworkBuildArg,
    FrameworkBuildArgs,
    FrameworkBuildSettings,
    FrameworkInfo,
    FrameworkSupport,
    read_config_files
};
use crate::commands::build::js_module::{get_call_string_property, get_string_property_value, is_call_ident};
use crate::doctavious_error::DoctaviousError;
use crate::doctavious_error::{Result as DoctaviousResult};

#[derive(Deserialize)]
struct VuePressConfig { dest: Option<String> }

pub struct VuePress { info: FrameworkInfo }

impl Default for VuePress{
    fn default() -> Self {
        Self {
            info: FrameworkInfo {
                name: "VuePress",
                website: Some("https://vuepress.vuejs.org/"),
                configs: Some(vec![
                    ".vuepress/config.js",
                    ".vuepress/config.yml",
                    ".vuepress/config.toml",
                    ".vuepress/config.ts"
                ]),
                project_file: None,
                build: FrameworkBuildSettings {
                    command: "vuepress build",
                    command_args: Some(FrameworkBuildArgs {
                        source: Some(FrameworkBuildArg::Arg(1, Some("docs"))),
                        config: Some(FrameworkBuildArg::Option {
                            short: "-c",
                            long: "--config"
                        }),
                        output: Some(FrameworkBuildArg::Option {
                            short: "-d",
                            long: "--dest"
                        })
                    }),
                    output_directory: ".vuepress/dist",
                },
            },
        }
    }
}


impl FrameworkSupport for VuePress {
    fn get_info(&self) -> &FrameworkInfo {
        &self.info
    }

    fn get_output_dir(&self) -> String {
        if let Some(configs) = &self.info.configs {
            match read_config_files::<VuePressConfig>(configs) {
                Ok(c) => {
                    if let Some(dest) = c.dest {
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

impl ConfigurationFileDeserialization for VuePressConfig {

    fn from_js_module(program: &Program) -> DoctaviousResult<Self> {
        // TODO: try and simplify
        if let Some(module) = program.as_module() {
            for item in &module.body {
                if let Some(ExportDefaultExpr(export_expression)) = item.as_module_decl() {
                    if let Some(call) = export_expression.expr.as_call() {
                        if is_call_ident(&call, "defineConfig") {
                            let dest = get_call_string_property(&call, "dest");
                            if dest.is_some() {
                                return Ok(Self {
                                    dest
                                });
                            }
                        }
                    }
                } else if let Some(Expr(stmt)) = item.as_stmt() {
                    let expression = &*stmt.expr;
                    if let Some(assign) = expression.as_assign() {
                        let rhs = &*assign.right;
                        if let Some(obj) = rhs.as_object() {
                            let dest = get_string_property_value(&obj.props, "dest");
                            if dest.is_some() {
                                return Ok(Self {
                                    dest
                                });
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
    use crate::commands::build::framework::{FrameworkBuildSettings, FrameworkInfo, FrameworkSupport};
    use super::VuePress;

    #[test]
    fn test_vuepress() {
        let configs = [
            "tests/resources/framework_configs/vuepress/config.js",
            "tests/resources/framework_configs/vuepress/config.toml",
            "tests/resources/framework_configs/vuepress/config.ts"
        ];
        for config in configs {
            let vuepress = VuePress {
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

            let output = vuepress.get_output_dir();
            assert_eq!(output, String::from("build"))
        }

    }

}

// next.config.js / next.config.mjs
// this is a regular Node.js module
// could also look at package.json -> scripts -> "build": "next build",

// .next -> default directory
// change be changed via distDir

use serde::{Deserialize};
use swc_ecma_ast::{Program};
use crate::commands::build::framework::{ConfigurationFileDeserialization, FrameworkBuildSettings, FrameworkDetectionItem, FrameworkDetector, FrameworkInfo, FrameworkMatchingStrategy, FrameworkSupport, read_config_files};
use crate::commands::build::js_module::{PropertyAccessor};
use crate::commands::build::language::Language;
use crate::doctavious_error::DoctaviousError;
use crate::doctavious_error::{Result as DoctaviousResult};

#[derive(Deserialize)]
struct NextJSConfig { output: String }

pub struct NextJS { info: FrameworkInfo }

impl NextJS {
    fn new(configs: Option<Vec<&'static str>>) -> Self {
        Self {
            info: FrameworkInfo {
                name: "Next.js",
                website: Some("https://nextjs.org/"),
                configs,
                language: Language::Javascript,
                detection: FrameworkDetector {
                    matching_strategy: FrameworkMatchingStrategy::Every,
                    detectors: vec![
                        FrameworkDetectionItem::Package {dependency: "next"}
                    ]
                },
                build: FrameworkBuildSettings {
                    command: "next build",
                    command_args: None,
                    output_directory: ".next",
                },
            }
        }
    }
}

impl Default for NextJS {
    fn default() -> Self {
        NextJS::new(Some(Vec::from(["next.config.js", "next.config.mjs"])))
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

        self.info.build.output_directory.to_string()
    }
}

impl ConfigurationFileDeserialization for NextJSConfig {

    fn from_js_module(program: &Program) -> DoctaviousResult<Self> {
        // TODO: try and simplify
        if let Some(module) = program.as_module() {
            if let Some(output) = module.get_property_as_string("distDir") {
                return Ok(Self {
                    output
                });
            }
            // for item in &module.body {
            //     if let Some(Decl(decl)) = item.as_stmt() {
            //         if let Some(variable_decl) = decl.as_var() {
            //             let variable = &**variable_decl;
            //             for declaration in &variable.decls {
            //                 if let Some(output) = get_variable_property_as_string(&declaration, "distDir") {
            //                     return Ok(Self {
            //                         output
            //                     });
            //                 }
            //             }
            //         }
            //     } else if let Some(Expr(stmt)) = item.as_stmt() {
            //         let expression = &*stmt.expr;
            //         if let Some(assign) = expression.as_assign() {
            //             let rhs = &*assign.right;
            //             if let Some(obj) = rhs.as_object() {
            //                 if let Some(output) = get_string_property_value(&obj.props, "distDir") {
            //                     return Ok(Self {
            //                         output
            //                     });
            //                 }
            //             }
            //         }
            //     }
            //
            // }
        }
        Err(DoctaviousError::Msg("invalid config".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::build::framework::{FrameworkSupport};
    use super::NextJS;

    #[test]
    fn test_nextjs() {
        for config in [
            "tests/resources/framework_configs/nextjs/next_js_v1.mjs",
            "tests/resources/framework_configs/nextjs/next_js_v2.mjs"
        ] {
            let nextjs = NextJS::new(Some(vec![config]));

            let output = nextjs.get_output_dir();
            assert_eq!(output, String::from("build"))
        }

    }

}

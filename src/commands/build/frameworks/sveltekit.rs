// defaults to ".svelte-kit"
// svelte.config.js
// outDir overrides
// dependency - adapter-static


use serde::{Serialize, Deserialize, de};
use swc_ecma_ast::{ArrayLit, Lit, ModuleDecl, ModuleItem, ObjectLit, Program, Stmt};
use swc_ecma_ast::Expr::{Array, Object, Tpl};
use swc_ecma_ast::Stmt::{Decl, Expr};

use crate::commands::build::frameworks::framework::{ConfigurationFileDeserialization, FrameworkBuildArg, FrameworkBuildArgs, FrameworkBuildOption, FrameworkBuildSettings, FrameworkInfo, FrameworkSupport, read_config_files};
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
                        output: Some(FrameworkBuildArg::Option(FrameworkBuildOption {
                            short: "",
                            long: "--outDir"
                        }))
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
        //self.info.build.outputdir
        String::default()
    }
}

impl ConfigurationFileDeserialization for SvelteKitConfig {
    fn from_js_module(program: &Program) -> DoctaviousResult<Self> {
        // TODO: try and simplify
        println!("{}", serde_json::to_string(program)?);
        if let Some(module) = program.as_module() {
            for item in &module.body {
                if let Some(Decl(decl)) = item.as_stmt() {
                    if let Some(variable_decl) = decl.as_var() {
                        for declaration in &variable_decl.decls {
                            if let Some(decl_ident) = declaration.name.as_ident() {
                                if decl_ident.sym.as_ref() != "config" {
                                    continue;
                                }
                            }
                            if let Some(init_decl) = &declaration.init {
                                if let Some(init_decl_obj) = init_decl.as_object() {
                                    for props in &init_decl_obj.props {
                                        if let Some(dir_prop) = props.as_prop() {
                                            if let Some(kv) = (*dir_prop).as_key_value() {
                                                if let Some(ident) = kv.key.as_ident() {
                                                    if ident.sym.as_ref() == "kit" {
                                                        if let Some(obj) = kv.value.as_object() {
                                                            for props in &obj.props {
                                                                if let Some(dir_prop) = props.as_prop() {
                                                                    if let Some(kv) = (*dir_prop).as_key_value() {
                                                                        if let Some(ident) = kv.key.as_ident() {
                                                                            if ident.sym.as_ref() == "outDir" {
                                                                                if let Some(Lit::Str(s)) = &kv.value.as_lit() {
                                                                                    return Ok(Self {
                                                                                        output: Some(s.value.to_string())
                                                                                    });
                                                                                } else if let Some(template) = kv.value.as_tpl() {
                                                                                    for q in &template.quasis {
                                                                                        // TODO: when would I not accept the first
                                                                                        if let Some(cooked) = &q.cooked {
                                                                                            return Ok(Self {
                                                                                                output: Some(cooked.to_string())
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

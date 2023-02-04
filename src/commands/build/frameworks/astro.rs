// astro.config.mjs
// "npm run build"
// astro build
// outDir: './my-custom-build-directory'
// defaults to "./dist"


use serde::{Serialize, Deserialize, de};
use swc_ecma_ast::{Lit, ModuleDecl, Program};
use crate::commands::build::frameworks::framework::{ConfigurationFileDeserialization, FrameworkBuildArg, FrameworkBuildArgs, FrameworkBuildOption, FrameworkBuildSettings, FrameworkInfo, FrameworkSupport, read_config_files};
use crate::doctavious_error::DoctaviousError;
use crate::DoctaviousResult;

pub struct Astro { info: FrameworkInfo }

impl Default for Astro {
    fn default() -> Self {
        Self {
            info: FrameworkInfo {
                name: "Astro",
                website: Some("https://astro.build"),
                configs: Some(Vec::from(["astro.config.mjs"])),
                project_file: None,
                build: FrameworkBuildSettings {
                    command: "astro build",
                    command_args: Some(FrameworkBuildArgs {
                        source: None,
                        config: Some(FrameworkBuildArg::Option(FrameworkBuildOption {
                            short: "",
                            long: "--config",
                        })),
                        output: None,
                    }),
                    output_directory: "./dist",
                },
            }
        }
    }
}

impl FrameworkSupport for Astro {
    fn get_info(&self) -> &FrameworkInfo {
        &self.info
    }

    fn get_output_dir(&self) -> String {
        if let Some(configs) = &self.info.configs {
            match read_config_files::<AstroConfig>(configs) {
                Ok(c) => {
                    return c.output
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

#[derive(Deserialize)]
struct AstroConfig { output: String }

impl ConfigurationFileDeserialization for AstroConfig {

    fn from_js_module(program: &Program) -> DoctaviousResult<Self> {
        // TODO: can this be simplified?
        if let Some(module) = program.as_module() {
            for item in &module.body {
                if let Some(ModuleDecl::ExportDefaultExpr(e)) = item.as_module_decl() {
                    let expression = &*e.expr;
                    if let Some(call) = expression.as_call() {
                        for call_args in &call.args {
                            let call_args_expression = &*call_args.expr;
                            if let Some(obj) = call_args_expression.as_object() {
                                for prop in &obj.props {
                                    if let Some(p) = prop.as_prop() {
                                        if let Some(kv) = (*p).as_key_value() {
                                            if let Some(ident) = kv.key.as_ident() {
                                                if ident.sym.as_ref() == "outDir" {
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
        Err(DoctaviousError::Msg("invalid config".to_string()))
    }
}



//
// pub fn from_program<'de, T>(p: Program) -> DoctaviousResult<T>
//     where
//         T: de::Deserialize<'de>,
// {
//     let c = T::try_from(p)?;
//     Ok(c)
// }

// impl From<Program> for AstroConfig {
//     fn from(_: Program) -> Self {
//         todo!()
//     }
// }

// impl TryFrom<Program> for AstroConfig {
//     type Error = DoctaviousError;
//
//     fn try_from(p: Program) -> Result<Self, Self::Error> {
//         if let Some(module) = p.module() {
//             for item in module.body {
//                 if let Some(ModuleDecl::ExportDefaultExpr(e)) = item.as_module_decl() {
//                     let expression = &*e.expr;
//                     if let Some(call) = &expression.as_call() {
//                         for call_args in &call.args {
//                             let call_args_expression = &*call_args.expr;
//                             if let Some(obj) = &call_args_expression.as_object() {
//                                 for prop in &obj.props {
//                                     if let Some(p) = &prop.as_prop() {
//                                         if let Some(kv) = (*p).as_key_value() {
//                                             if let Some(ident) = &kv.key.as_ident() {
//                                                 if ident.sym.to_string() == "outDir" {
//                                                     if let Some(Lit::Str(s)) = &kv.value.as_lit() {
//                                                         return Ok(AstroConfig {
//                                                             output: s.value.to_string()
//                                                         });
//                                                     }
//                                                 }
//                                             }
//                                         }
//                                     }
//                                 }
//                             }
//                         }
//                     }
//                 }
//             }
//         }
//         Err(DoctaviousError::Msg("invalid config".to_string()))
//         //         if TypeId::of::<ModuleDecl::ExportDefaultExpr>() == item.type_id() {
//         //
//         //         }
//         //         match item {
//         //             ModuleItem::ModuleDecl(d) => {
//         //                 match d {
//         //                     ModuleDecl::Import(_) => {}
//         //                     ModuleDecl::ExportDecl(_) => {}
//         //                     ModuleDecl::ExportNamed(_) => {}
//         //                     ModuleDecl::ExportDefaultDecl(_) => {}
//         //                     ModuleDecl::ExportDefaultExpr(e) => {
//         //                         match *e.expr {
//         //                             Expr::This(_) => {}
//         //                             Expr::Array(_) => {}
//         //                             Expr::Object(_) => {}
//         //                             Expr::Fn(_) => {}
//         //                             Expr::Unary(_) => {}
//         //                             Expr::Update(_) => {}
//         //                             Expr::Bin(_) => {}
//         //                             Expr::Assign(_) => {}
//         //                             Expr::Member(_) => {}
//         //                             Expr::SuperProp(_) => {}
//         //                             Expr::Cond(_) => {}
//         //                             Expr::Call(c) => {
//         //                                 for a in c.args {
//         //                                     match *a.expr {
//         //                                         Expr::This(_) => {}
//         //                                         Expr::Array(_) => {}
//         //                                         Expr::Object(o) => {
//         //                                             for prop in o.props {
//         //                                                 match prop {
//         //                                                     PropOrSpread::Spread(_) => {}
//         //                                                     PropOrSpread::Prop(p) => {
//         //                                                         match *p {
//         //                                                             Prop::Shorthand(_) => {}
//         //                                                             Prop::KeyValue(kv) => {
//         //                                                                 if let Some(ident) = kv.key.as_ident() {
//         //                                                                     if ident.sym.to_string() == "outDir" {
//         //                                                                         if let Some(val) = kv.value.lit() {
//         //                                                                             match val {
//         //                                                                                 Lit::Str(s) => {
//         //                                                                                     s.value.to_string()
//         //                                                                                 }
//         //                                                                                 Lit::Bool(_) => {}
//         //                                                                                 Lit::Null(_) => {}
//         //                                                                                 Lit::Num(_) => {}
//         //                                                                                 Lit::BigInt(_) => {}
//         //                                                                                 Lit::Regex(_) => {}
//         //                                                                                 Lit::JSXText(_) => {}
//         //                                                                             }
//         //                                                                         }
//         //                                                                     }
//         //                                                                 }
//         //                                                             }
//         //                                                             Prop::Assign(_) => {}
//         //                                                             Prop::Getter(_) => {}
//         //                                                             Prop::Setter(_) => {}
//         //                                                             Prop::Method(_) => {}
//         //                                                         }
//         //                                                     }
//         //                                                 }
//         //                                             }
//         //                                         }
//         //                                         Expr::Fn(_) => {}
//         //                                         Expr::Unary(_) => {}
//         //                                         Expr::Update(_) => {}
//         //                                         Expr::Bin(_) => {}
//         //                                         Expr::Assign(_) => {}
//         //                                         Expr::Member(_) => {}
//         //                                         Expr::SuperProp(_) => {}
//         //                                         Expr::Cond(_) => {}
//         //                                         Expr::Call(_) => {}
//         //                                         Expr::New(_) => {}
//         //                                         Expr::Seq(_) => {}
//         //                                         Expr::Ident(_) => {}
//         //                                         Expr::Lit(_) => {}
//         //                                         Expr::Tpl(_) => {}
//         //                                         Expr::TaggedTpl(_) => {}
//         //                                         Expr::Arrow(_) => {}
//         //                                         Expr::Class(_) => {}
//         //                                         Expr::Yield(_) => {}
//         //                                         Expr::MetaProp(_) => {}
//         //                                         Expr::Await(_) => {}
//         //                                         Expr::Paren(_) => {}
//         //                                         Expr::JSXMember(_) => {}
//         //                                         Expr::JSXNamespacedName(_) => {}
//         //                                         Expr::JSXEmpty(_) => {}
//         //                                         Expr::JSXElement(_) => {}
//         //                                         Expr::JSXFragment(_) => {}
//         //                                         Expr::TsTypeAssertion(_) => {}
//         //                                         Expr::TsConstAssertion(_) => {}
//         //                                         Expr::TsNonNull(_) => {}
//         //                                         Expr::TsAs(_) => {}
//         //                                         Expr::TsInstantiation(_) => {}
//         //                                         Expr::TsSatisfies(_) => {}
//         //                                         Expr::PrivateName(_) => {}
//         //                                         Expr::OptChain(_) => {}
//         //                                         Expr::Invalid(_) => {}
//         //                                     }
//         //                                 }
//         //                             }
//         //                             Expr::New(_) => {}
//         //                             Expr::Seq(_) => {}
//         //                             Expr::Ident(_) => {}
//         //                             Expr::Lit(_) => {}
//         //                             Expr::Tpl(_) => {}
//         //                             Expr::TaggedTpl(_) => {}
//         //                             Expr::Arrow(_) => {}
//         //                             Expr::Class(_) => {}
//         //                             Expr::Yield(_) => {}
//         //                             Expr::MetaProp(_) => {}
//         //                             Expr::Await(_) => {}
//         //                             Expr::Paren(_) => {}
//         //                             Expr::JSXMember(_) => {}
//         //                             Expr::JSXNamespacedName(_) => {}
//         //                             Expr::JSXEmpty(_) => {}
//         //                             Expr::JSXElement(_) => {}
//         //                             Expr::JSXFragment(_) => {}
//         //                             Expr::TsTypeAssertion(_) => {}
//         //                             Expr::TsConstAssertion(_) => {}
//         //                             Expr::TsNonNull(_) => {}
//         //                             Expr::TsAs(_) => {}
//         //                             Expr::TsInstantiation(_) => {}
//         //                             Expr::TsSatisfies(_) => {}
//         //                             Expr::PrivateName(_) => {}
//         //                             Expr::OptChain(_) => {}
//         //                             Expr::Invalid(_) => {}
//         //                         }
//         //                     }
//         //                     ModuleDecl::ExportAll(_) => {}
//         //                     ModuleDecl::TsImportEquals(_) => {}
//         //                     ModuleDecl::TsExportAssignment(_) => {}
//         //                     ModuleDecl::TsNamespaceExport(_) => {}
//         //                 }
//         //             }
//         //             ModuleItem::Stmt(_) => {}
//         //         }
//         //     }
//         // } else {
//         //     // error
//         // }
//
//         // Ok(AstroConfig {
//         //     output: String::default()
//         // })
//     }
// }

#[cfg(test)]
mod tests {
    use crate::commands::build::frameworks::framework::{FrameworkBuildSettings, FrameworkInfo, FrameworkSupport};
    use super::Astro;

    #[test]
    fn test_astro() {
        let astro = Astro {
            info: FrameworkInfo {
                name: "",
                website: None,
                configs: Some(vec!["tests/resources/framework_configs/astro/astro.config.mjs"]),
                project_file: None,
                build: FrameworkBuildSettings {
                    command: "",
                    command_args: None,
                    output_directory: "",
                },
            },
        };

        let output = astro.get_output_dir();
        assert_eq!(output, "./build")
    }

}

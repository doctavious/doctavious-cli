/// This contains a set of helper functions for traversing JS program modules

use swc_ecma_ast::{Expr, Lit, Program, PropOrSpread, TplElement, VarDeclarator};
use swc_ecma_ast::Stmt::{Decl};

pub(crate) fn get_variable_declaration<'a>(program: &'a Program, variable: &'static str) -> Option<&'a VarDeclarator> {
    if let Some(module) = program.as_module() {
        for item in &module.body {
            if let Some(Decl(decl)) = item.as_stmt() {
                if let Some(variable_decl) = decl.as_var() {
                    for declaration in &variable_decl.decls {
                        if let Some(decl_ident) = declaration.name.as_ident() {
                            if decl_ident.sym.as_ref() == variable {
                                return Some(declaration);
                            }
                        }
                    }
                }
            }
        }
    }
    None
}


pub(crate) fn get_variable_properties<'a>(variable: &'a VarDeclarator, property: &'static str) -> Option<&'a Vec<PropOrSpread>> {
    if let Some(init_decl) = &variable.init {
        if let Some(init_decl_obj) = init_decl.as_object() {
            for prop_spread in &init_decl_obj.props {
                if let Some(prop) = prop_spread.as_prop() {
                    if let Some(kv) = (*prop).as_key_value() {
                        if let Some(ident) = kv.key.as_ident() {
                            if ident.sym.as_ref() == property {
                                if let Some(obj) = kv.value.as_object() {
                                    return Some(&obj.props);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

pub(crate) fn get_string_property_value(properties: &Vec<PropOrSpread>, key: &'static str) -> Option<String> {
    for prop_spread in properties {
        if let Some(prop) = prop_spread.as_prop() {
            if let Some(kv) = (*prop).as_key_value() {
                if let Some(ident) = kv.key.as_ident() {
                    if ident.sym.as_ref() == key {
                        return match &*kv.value {
                            Expr::Lit(l) => {
                                match l {
                                    Lit::Str(v) => Some(v.value.to_string()),
                                    Lit::Bool(v) => Some(v.value.to_string()),
                                    Lit::Null(_) => None,
                                    Lit::Num(v) => Some(v.to_string()),
                                    Lit::BigInt(v) => Some(v.value.to_string()),
                                    Lit::Regex(v) => Some(v.exp.to_string()),
                                    Lit::JSXText(v) => Some(v.value.to_string())
                                }
                            }
                            Expr::Tpl(tpl) => {
                                return if let Some(TplElement { cooked: Some(atom), .. }) = &tpl.quasis.first() {
                                    Some(atom.to_string())
                                } else {
                                    None
                                }
                            }
                            _ => None
                        };
                    }
                }
            }
        }
    }

    None
}

/// This contains a set of helper functions for traversing JS program modules

use swc_ecma_ast::{Expr, Lit, Program, ModuleDecl, PropOrSpread, TplElement, VarDeclarator, CallExpr, Function, ObjectLit, ArrayLit, Prop, KeyValueProp};
use swc_ecma_ast::Stmt::{Decl, Expr as ExprStmt};

// TODO: maybe create some traits for these

pub(crate) fn get_variable_declaration<'a>(program: &'a Program, ident: &'static str) -> Option<&'a VarDeclarator> {
    if let Some(module) = program.as_module() {
        for item in &module.body {
            if let Some(Decl(decl)) = item.as_stmt() {
                if let Some(variable_decl) = decl.as_var() {
                    for declaration in &variable_decl.decls {
                        if let Some(decl_ident) = declaration.name.as_ident() {
                            if decl_ident.sym.as_ref() == ident {
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

pub(crate) fn get_call_expression<'a>(program: &'a Program, ident: &'static str) -> Option<&'a CallExpr> {
    if let Some(module) = program.as_module() {
        for item in &module.body {
            if let Some(ModuleDecl::ExportDefaultExpr(e)) = item.as_module_decl() {
                let expression = &*e.expr;
                if let Some(call) = expression.as_call() {
                    if let Some(callee) = call.callee.as_expr() {
                        if let Some(callee_ident) = callee.as_ident() {
                            if callee_ident.sym.as_ref() == ident {
                                return Some(&call);
                            }
                        }
                    }
                }
            }
        }
    }
    None
}


pub(crate) fn get_assignment_function(program: &Program) -> Option<&Function> {
    if let Some(module) = program.as_module() {
        for item in &module.body {
            if let Some(ExprStmt(stmt)) = item.as_stmt() {
                let expression = &*stmt.expr;
                if let Some(assign) = expression.as_assign() {
                    let rhs = &*assign.right;
                    if let Some(func) = rhs.as_fn_expr() {
                        return Some(&*func.function);
                    }
                }
            }
        }
    }
    None
}

pub(crate) fn get_assignment_obj(program: &Program) -> Option<&ObjectLit> {
    if let Some(module) = program.as_module() {
        for item in &module.body {
            if let Some(ExprStmt(stmt)) = item.as_stmt() {
                let expression = &*stmt.expr;
                if let Some(assign) = expression.as_assign() {
                    let rhs = &*assign.right;
                    let obj = match rhs {
                        Expr::Object(o) =>  Some(o),
                        Expr::Fn(f) => get_function_return_obj(&f.function),
                        _ => None
                    };
                    if obj.is_some() {
                        return obj;
                    }
                }
            }
        }
    }
    None
}

pub(crate) fn get_function_return_obj(func: &Function) -> Option<&ObjectLit> {
    if let Some(fn_body) = &func.body {
        for statement in &fn_body.stmts {
            if let Some(return_statement) = statement.as_return_stmt() {
                if let Some(return_statement_args) = &return_statement.arg {
                    let args = &**return_statement_args;
                    return args.as_object();
                }
            }
        }
    }

    None
}

pub(crate) fn get_call_string_property(call: &CallExpr, property: &'static str) -> Option<String> {
    for call_args in &call.args {
        let call_args_expression = &*call_args.expr;
        if let Some(obj) = call_args_expression.as_object() {
            return get_string_property_value(&obj.props, property);
        }
    }

    None
}

// TODO: can this be generic
// tuple enum variants (and also tuple structs) can be called like functions, so they implement Fn traits and can be passed where a function is expected.
// where F: 'static + Fn(String) -> Message
pub(crate) fn get_obj_property<'a>(obj: &'a ObjectLit, property_ident: &'static str) -> Option<&'a ObjectLit> {
    for prop in &obj.props {
        if let Some(p) = prop.as_prop() {
            if let Some(kv) = (*p).as_key_value() {
                if let Some(key_ident) = kv.key.as_ident() {
                    if key_ident.sym.as_ref() == property_ident {
                        return kv.value.as_object();
                    }
                }
            }
        }
    }

    None
}

pub(crate) fn get_array_property<'a>(obj: &'a ObjectLit, property_ident: &'static str) -> Option<&'a ArrayLit> {
    for prop in &obj.props {
        if let Some(p) = prop.as_prop() {
            if let Some(kv) = (*p).as_key_value() {
                if let Some(key_ident) = kv.key.as_ident() {
                    if key_ident.sym.as_ref() == property_ident {
                        return kv.value.as_array();
                    }
                }
            }
        }
    }

    None
}

pub(crate) fn find_array_element<'a>(
    arr: &'a ArrayLit,
    property_ident: &'static str,
    val: &'static str
) -> Option<&'a ObjectLit> {
    for elem in &arr.elems {
        if let Some(exp) = elem {
            if let Some(o) = exp.expr.as_object() {
                for op in &o.props {
                    if let Some(op2) = op.as_prop() {
                        if let Some(kv) = op2.as_key_value() {
                            if let Some(ident) = kv.key.as_ident() {
                                if ident.sym.as_ref() == property_ident {
                                    if let Some(kv_val) = get_value_from_kv_as_string(kv) {
                                        if kv_val == val {
                                            return Some(o);
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
    None
}

// TODO: add method that just gets specific string via get_string_property_value
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

pub(crate) fn get_variable_property_as_string(variable: &VarDeclarator, property: &'static str) -> Option<String> {
    if let Some(init_decl) = &variable.init {
        if let Some(init_decl_obj) = init_decl.as_object() {
            return get_string_property_value(&init_decl_obj.props, property);
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
                        let val = get_value_from_kv_as_string(kv);
                        if val.is_some() {
                            return val;
                        }
                    }
                }
            }

            // if let Some(kv) = (*prop).as_key_value() {
            //     if let Some(ident) = kv.key.as_ident() {
            //         if ident.sym.as_ref() == key {
            //             return match &*kv.value {
            //                 Expr::Lit(l) => {
            //                     match l {
            //                         Lit::Str(v) => Some(v.value.to_string()),
            //                         Lit::Bool(v) => Some(v.value.to_string()),
            //                         Lit::Null(_) => None,
            //                         Lit::Num(v) => Some(v.to_string()),
            //                         Lit::BigInt(v) => Some(v.value.to_string()),
            //                         Lit::Regex(v) => Some(v.exp.to_string()),
            //                         Lit::JSXText(v) => Some(v.value.to_string())
            //                     }
            //                 }
            //                 Expr::Tpl(tpl) => {
            //                     return if let Some(TplElement { cooked: Some(atom), .. }) = &tpl.quasis.first() {
            //                         Some(atom.to_string())
            //                     } else {
            //                         None
            //                     }
            //                 }
            //                 _ => None
            //             };
            //         }
            //     }
            // }
        }
    }

    None
}

pub(crate) fn get_string_from_property(prop: &Prop) -> Option<String> {
    if let Some(kv) = (*prop).as_key_value() {
        return get_value_from_kv_as_string(kv);
    }

    None
}


pub(crate) fn get_value_from_kv_as_string(kv: &KeyValueProp) -> Option<String> {
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

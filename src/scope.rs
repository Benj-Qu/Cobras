use core::panic;
use std::collections::HashMap;

use crate::compile::CompileErr;
use crate::syntax::{Exp, FunDecl, SurfProg};

static MAX_SNAKE_INT: i64 = i64::MAX >> 1;
static MIN_SNAKE_INT: i64 = i64::MIN >> 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parameters {
    pub local: Vec<String>,
    pub global: Vec<Exp<()>>,
}

pub fn get<T>(env: &[(&str, T)], x: &str) -> Option<T>
where
    T: Clone,
{
    for (y, n) in env.iter().rev() {
        if x == *y {
            return Some((*n).clone());
        }
    }
    None
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VarType {
    Var,
    Field,
}

pub fn check_prog<'exp, Span>(
    p: &'exp SurfProg<Span>,
    mut var_env: Vec<(&'exp str, VarType)>,
    mut class_env: Vec<(&'exp str, usize)>,
    mut method_env: Vec<(&'exp str, ())>,
) -> Result<(), CompileErr<Span>>
where
    Span: Clone,
{
    match p {
        Exp::Num(i, ann) => {
            // Check Overflow
            if *i <= MAX_SNAKE_INT && *i >= MIN_SNAKE_INT {
                Ok(())
            } else {
                Err(CompileErr::Overflow {
                    num: *i,
                    location: ann.clone(),
                })
            }
        }
        Exp::Bool(..) => Ok(()),
        Exp::Var(x, ann) => {
            // Check UnboundVariable
            match get(&var_env, &x) {
                Some(_) => Ok(()),
                None => Err(CompileErr::UnboundVariable {
                    unbound: x.clone(),
                    location: ann.clone(),
                }),
            }
        }
        Exp::Prim1(_, p, _) => {
            check_prog(p, var_env.clone(), class_env.clone(), method_env.clone())
        }
        Exp::Prim2(_, p1, p2, _) => {
            match check_prog(p1, var_env.clone(), class_env.clone(), method_env.clone()) {
                Ok(()) => check_prog(p2, var_env.clone(), class_env.clone(), method_env.clone()),
                Err(msg) => Err(msg),
            }
        }
        Exp::Let {
            bindings,
            body,
            ann,
        } => {
            // Check DuplicateBinding
            let mut env_temp: Vec<(&str, ())> = Vec::new();
            for (x, _) in bindings.iter() {
                match get(&env_temp, &x) {
                    Some(()) => {
                        return Err(CompileErr::DuplicateBinding {
                            duplicated_name: x.clone(),
                            location: ann.clone(),
                        })
                    }
                    None => env_temp.push((&x, ())),
                };
            }
            // Append variables to the environment
            for (x, p) in bindings.iter() {
                // Check binding definition
                match check_prog(p, var_env.clone(), class_env.clone(), method_env.clone()) {
                    Ok(()) => (),
                    Err(msg) => return Err(msg),
                };
                var_env.push((&x, VarType::Var));
            }
            check_prog(body, var_env.clone(), class_env.clone(), method_env.clone())
        }
        Exp::If {
            cond,
            thn,
            els,
            ann: _,
        } => match check_prog(cond, var_env.clone(), class_env.clone(), method_env.clone()) {
            Ok(()) => match check_prog(thn, var_env.clone(), class_env.clone(), method_env.clone())
            {
                Ok(()) => check_prog(els, var_env.clone(), class_env.clone(), method_env.clone()),
                Err(msg) => Err(msg),
            },
            Err(msg) => Err(msg),
        },
        Exp::Array(array, _) => {
            for element in array.iter() {
                match check_prog(
                    element,
                    var_env.clone(),
                    class_env.clone(),
                    method_env.clone(),
                ) {
                    Ok(()) => (),
                    Err(msg) => return Err(msg),
                };
            }
            Ok(())
        }
        Exp::ArraySet {
            array,
            index,
            new_value,
            ann: _,
        } => match check_prog(
            array,
            var_env.clone(),
            class_env.clone(),
            method_env.clone(),
        ) {
            Ok(()) => match check_prog(
                index,
                var_env.clone(),
                class_env.clone(),
                method_env.clone(),
            ) {
                Ok(()) => check_prog(
                    new_value,
                    var_env.clone(),
                    class_env.clone(),
                    method_env.clone(),
                ),
                Err(msg) => Err(msg),
            },
            Err(msg) => Err(msg),
        },
        Exp::Semicolon { e1, e2, ann: _ } => {
            match check_prog(e1, var_env.clone(), class_env.clone(), method_env.clone()) {
                Ok(()) => check_prog(e2, var_env.clone(), class_env.clone(), method_env.clone()),
                Err(msg) => Err(msg),
            }
        }
        Exp::FunDefs {
            decls,
            body,
            ann: _,
        } => {
            let mut env_temp_fun = Vec::new();
            for FunDecl {
                name,
                parameters,
                body: _,
                ann,
            } in decls.iter()
            {
                let mut env_temp_arg = Vec::new();
                // Check DuplicateArgName
                for arg in parameters.iter() {
                    match get(&env_temp_arg, &arg) {
                        Some(_) => {
                            return Err(CompileErr::DuplicateArgName {
                                duplicated_name: arg.clone(),
                                location: ann.clone(),
                            })
                        }
                        None => env_temp_arg.push((&arg, ())),
                    }
                }
                // Check DuplicateFunName
                match get(&env_temp_fun, &name) {
                    Some(_) => {
                        return Err(CompileErr::DuplicateFunName {
                            duplicated_name: name.clone(),
                            location: ann.clone(),
                        })
                    }
                    None => env_temp_fun.push((&name, ())),
                }
                // Append functions to the environment
                var_env.push((&name, VarType::Var));
            }
            // Check function definitions
            for FunDecl {
                parameters, body, ..
            } in decls.iter()
            {
                let mut env_clone = var_env.clone();
                for arg in parameters.iter() {
                    env_clone.push((&arg, VarType::Var));
                }
                match check_prog(body, env_clone, class_env.clone(), method_env.clone()) {
                    Ok(()) => (),
                    Err(msg) => return Err(msg),
                };
            }
            check_prog(body, var_env.clone(), class_env.clone(), method_env.clone())
        }
        Exp::Call(fun, args, _) => {
            // Check args
            for arg in args.iter() {
                match check_prog(arg, var_env.clone(), class_env.clone(), method_env.clone()) {
                    Ok(()) => (),
                    Err(msg) => return Err(msg),
                };
            }
            check_prog(fun, var_env.clone(), class_env.clone(), method_env.clone())
        }
        Exp::Lambda {
            parameters,
            body,
            ann,
        } => {
            let mut env_temp_arg = Vec::new();
            // Check DuplicateArgName
            for arg in parameters.iter() {
                match get(&env_temp_arg, &arg) {
                    Some(_) => {
                        return Err(CompileErr::DuplicateArgName {
                            duplicated_name: arg.clone(),
                            location: ann.clone(),
                        })
                    }
                    None => env_temp_arg.push((&arg, ())),
                }
            }
            for arg in parameters.iter() {
                var_env.push((&arg, VarType::Var));
            }
            check_prog(body, var_env.clone(), class_env.clone(), method_env.clone())
        }
        Exp::ClassDef {
            name,
            fields,
            methods,
            body,
            ann,
        } => {
            // Add class to environment
            class_env.push((&name, fields.len()));
            // Check DuplicateField
            let mut env_temp_field: Vec<(&'exp str, ())> = Vec::new();
            for field in fields.iter() {
                match get(&env_temp_field, &field) {
                    Some(_) => {
                        return Err(CompileErr::DuplicateField {
                            duplicated_name: field.clone(),
                            location: ann.clone(),
                        })
                    }
                    None => env_temp_field.push((&field, ())),
                }
            }
            // Check DuplicateMethod
            let mut env_temp_method: Vec<(&'exp str, ())> = Vec::new();
            for method in methods.iter() {
                match get(&env_temp_field, &method.name) {
                    Some(_) => {
                        return Err(CompileErr::DuplicateMethod {
                            duplicated_name: method.name.clone(),
                            location: ann.clone(),
                        })
                    }
                    None => env_temp_method.push((&method.name, ())),
                }
            }
            // Construct a temp environment for the methods
            let mut var_env_clone: Vec<(&'exp str, VarType)> = var_env.clone();
            for field in fields.iter() {
                var_env_clone.push((&field, VarType::Field));
            }
            for method in methods.iter() {
                var_env_clone.push((&method.name, VarType::Field));
            }
            // Append methods to the environment
            for method in methods.iter() {
                method_env.push((&method.name, ()))
            }
            // Check methods
            // Check DuplicateArgName
            for FunDecl {
                parameters, ann, ..
            } in methods.iter()
            {
                let mut env_temp_arg = Vec::new();
                for arg in parameters.iter() {
                    match get(&env_temp_arg, &arg) {
                        Some(_) => {
                            return Err(CompileErr::DuplicateArgName {
                                duplicated_name: arg.clone(),
                                location: ann.clone(),
                            })
                        }
                        None => env_temp_arg.push((&arg, ())),
                    }
                }
            }
            // Check method body
            for FunDecl {
                parameters, body, ..
            } in methods.iter()
            {
                let mut env_clone = var_env_clone.clone();
                for arg in parameters.iter() {
                    env_clone.push((&arg, VarType::Var));
                }
                match check_prog(
                    body,
                    env_clone.clone(),
                    class_env.clone(),
                    method_env.clone(),
                ) {
                    Ok(()) => (),
                    Err(msg) => return Err(msg),
                };
            }
            // Check body
            check_prog(body, var_env.clone(), class_env.clone(), method_env.clone())
        }
        Exp::Object { class, fields, ann } => {
            // Check fields
            for field in fields.iter() {
                match check_prog(
                    field,
                    var_env.clone(),
                    class_env.clone(),
                    method_env.clone(),
                ) {
                    Ok(()) => (),
                    Err(msg) => return Err(msg),
                };
            }
            // Check UndefinedClass
            match get(&class_env, &class) {
                Some(len) => {
                    if len == fields.len() {
                        Ok(())
                    } else {
                        Err(CompileErr::WrongFieldSize {
                            class: class.clone(),
                            location: ann.clone(),
                        })
                    }
                }
                None => Err(CompileErr::UndefinedClass {
                    undefined: class.clone(),
                    location: ann.clone(),
                }),
            }
        }
        Exp::CallMethod {
            object,
            method,
            args,
            ann,
        } => {
            // Check args
            for arg in args.iter() {
                match check_prog(arg, var_env.clone(), class_env.clone(), method_env.clone()) {
                    Ok(()) => (),
                    Err(msg) => return Err(msg),
                };
            }
            // check object
            match check_prog(
                object,
                var_env.clone(),
                class_env.clone(),
                method_env.clone(),
            ) {
                Ok(()) => (),
                Err(msg) => return Err(msg),
            }
            // Check UndefinedMethod
            match get(&method_env, &method) {
                Some(()) => Ok(()),
                None => Err(CompileErr::UndefinedMethod {
                    undefined: method.clone(),
                    location: ann.clone(),
                }),
            }
        }
        Exp::SetField { field, value, ann } => {
            // Check value
            match check_prog(
                value,
                var_env.clone(),
                class_env.clone(),
                method_env.clone(),
            ) {
                Ok(()) => (),
                Err(msg) => return Err(msg),
            };
            // Check UndefinedField
            match get(&var_env, &field) {
                Some(VarType::Field) => Ok(()),
                _ => Err(CompileErr::UndefinedField {
                    undefined: field.clone(),
                    location: ann.clone(),
                }),
            }
        }
        Exp::MakeClosure { .. } | Exp::CallUniqMethod { .. } | Exp::MethodDefs { .. } => {
            panic!("Should never exist during check prog!")
        }
    }
}

pub fn uniquify<'exp>(
    e: &'exp SurfProg<u32>,
    mut var_env: Vec<(&'exp str, String)>,
    mut class_env: Vec<(&'exp str, String)>,
    mut method_env: HashMap<&'exp str, HashMap<String, String>>,
) -> Exp<()> {
    match e {
        Exp::Num(i, _) => Exp::Num(*i, ()),
        Exp::Bool(b, _) => Exp::Bool(*b, ()),
        Exp::Var(x, _) => {
            let uniq_id = match get(&var_env, &x) {
                Some(x) => x,
                None => panic!("Variable is guaranteed to be in scope"),
            };
            Exp::Var(uniq_id.to_string(), ())
        }
        Exp::Prim1(prim, e, _) => Exp::Prim1(
            *prim,
            Box::new(uniquify(
                &e,
                var_env.clone(),
                class_env.clone(),
                method_env.clone(),
            )),
            (),
        ),
        Exp::Prim2(prim, e1, e2, _) => Exp::Prim2(
            *prim,
            Box::new(uniquify(
                &e1,
                var_env.clone(),
                class_env.clone(),
                method_env.clone(),
            )),
            Box::new(uniquify(
                &e2,
                var_env.clone(),
                class_env.clone(),
                method_env.clone(),
            )),
            (),
        ),
        Exp::Let {
            bindings,
            body,
            ann,
        } => {
            let mut uniq_bindings: Vec<(String, Exp<()>)> = Vec::new();
            for (x, e) in bindings.iter() {
                let uniq_id = format!("#{}_{}", x, ann);
                uniq_bindings.push((
                    uniq_id.clone(),
                    uniquify(&e, var_env.clone(), class_env.clone(), method_env.clone()),
                ));
                var_env.push((&x, uniq_id.clone()));
            }
            Exp::Let {
                bindings: uniq_bindings,
                body: Box::new(uniquify(
                    &body,
                    var_env.clone(),
                    class_env.clone(),
                    method_env.clone(),
                )),
                ann: (),
            }
        }
        Exp::If {
            cond,
            thn,
            els,
            ann: _,
        } => Exp::If {
            cond: Box::new(uniquify(
                &cond,
                var_env.clone(),
                class_env.clone(),
                method_env.clone(),
            )),
            thn: Box::new(uniquify(
                &thn,
                var_env.clone(),
                class_env.clone(),
                method_env.clone(),
            )),
            els: Box::new(uniquify(
                &els,
                var_env.clone(),
                class_env.clone(),
                method_env.clone(),
            )),
            ann: (),
        },
        Exp::Array(array, _) => {
            let uniq_array: Vec<Exp<()>> = array
                .iter()
                .map(|element: &Exp<u32>| {
                    uniquify(
                        element,
                        var_env.clone(),
                        class_env.clone(),
                        method_env.clone(),
                    )
                })
                .collect();
            Exp::Array(uniq_array, ())
        }
        Exp::ArraySet {
            array,
            index,
            new_value,
            ann: _,
        } => Exp::ArraySet {
            array: Box::new(uniquify(
                &array,
                var_env.clone(),
                class_env.clone(),
                method_env.clone(),
            )),
            index: Box::new(uniquify(
                &index,
                var_env.clone(),
                class_env.clone(),
                method_env.clone(),
            )),
            new_value: Box::new(uniquify(
                &new_value,
                var_env.clone(),
                class_env.clone(),
                method_env.clone(),
            )),
            ann: (),
        },
        Exp::Semicolon { e1, e2, ann: _ } => Exp::Semicolon {
            e1: Box::new(uniquify(
                &e1,
                var_env.clone(),
                class_env.clone(),
                method_env.clone(),
            )),
            e2: Box::new(uniquify(
                &e2,
                var_env.clone(),
                class_env.clone(),
                method_env.clone(),
            )),
            ann: (),
        },
        Exp::FunDefs {
            decls,
            body,
            ann: _,
        } => {
            for FunDecl { name, ann, .. } in decls.iter() {
                var_env.push((&name, format!("{}_{}", name, ann)));
            }
            let uniq_decls: Vec<FunDecl<Exp<()>, ()>> = decls
                .iter()
                .map(|fun: &FunDecl<Exp<u32>, u32>| {
                    let uniq_fun: String = match get(&var_env, &fun.name) {
                        Some(x) => x,
                        None => panic!("Function is guaranteed to be in scope"),
                    };
                    let mut env_clone: Vec<(&str, String)> = var_env.clone();
                    let mut uniq_args: Vec<String> = Vec::new();
                    for arg in fun.parameters.iter() {
                        let uniq_arg: String = format!("#{}_{}", arg, fun.ann);
                        env_clone.push((&arg, uniq_arg.clone()));
                        uniq_args.push(uniq_arg);
                    }
                    FunDecl {
                        name: uniq_fun.to_string(),
                        parameters: uniq_args,
                        body: uniquify(&fun.body, env_clone, class_env.clone(), method_env.clone()),
                        ann: (),
                    }
                })
                .collect();
            Exp::FunDefs {
                decls: uniq_decls,
                body: Box::new(uniquify(
                    &body,
                    var_env.clone(),
                    class_env.clone(),
                    method_env.clone(),
                )),
                ann: (),
            }
        }
        Exp::Call(fun, args, _) => {
            let uniq_args: Vec<Exp<()>> = args
                .iter()
                .map(|arg: &Exp<u32>| {
                    uniquify(arg, var_env.clone(), class_env.clone(), method_env.clone())
                })
                .collect();
            Exp::Call(
                Box::new(uniquify(
                    &fun,
                    var_env.clone(),
                    class_env.clone(),
                    method_env.clone(),
                )),
                uniq_args,
                (),
            )
        }
        Exp::Lambda {
            parameters,
            body,
            ann,
        } => {
            let mut uniq_parameters: Vec<String> = Vec::new();
            for parameter in parameters.iter() {
                let uniq_parameter: String = format!("#{}_{}", parameter, ann);
                var_env.push((&parameter, uniq_parameter.clone()));
                uniq_parameters.push(uniq_parameter);
            }
            let uniq_name: String = format!("Lambda_{}", ann);
            let uniq_decls: Vec<FunDecl<Exp<()>, ()>> = vec![FunDecl {
                name: uniq_name.clone(),
                parameters: uniq_parameters.clone(),
                body: uniquify(
                    &body,
                    var_env.clone(),
                    class_env.clone(),
                    method_env.clone(),
                ),
                ann: (),
            }];
            Exp::FunDefs {
                decls: uniq_decls,
                body: Box::new(Exp::Var(uniq_name.clone(), ())),
                ann: (),
            }
        }
        Exp::ClassDef {
            name,
            fields,
            methods,
            body,
            ann,
        } => {
            let uniq_name: String = format!("{}_{}", name, ann);
            class_env.push((&name, uniq_name.clone()));
            let mut uniq_fields: Vec<String> = Vec::new();
            let var_env_clone: Vec<(&'exp str, String)> = var_env.clone();
            // Update variable environment for method body
            for field in fields.iter() {
                let uniq_field: String = format!("#{}_{}", uniq_name, field);
                var_env.push((&field, uniq_field.clone()));
                uniq_fields.push(uniq_field);
            }
            for method in methods.iter() {
                let uniq_method_name = format!("{}_{}_{}", uniq_name, method.name, method.ann);
                var_env.push((&method.name, uniq_method_name.clone()));
            }
            let mut uniq_methods: Vec<FunDecl<Exp<()>, ()>> = Vec::new();
            for method in methods.iter() {
                let uniq_method_name = format!("{}_{}_{}", uniq_name, method.name, method.ann);
                match method_env.get(method.name.as_str()) {
                    Some(tbl) => {
                        let mut tbl_update = tbl.clone();
                        tbl_update.insert(uniq_name.clone(), uniq_method_name.clone());
                        method_env.insert(method.name.as_str(), tbl_update);
                    }
                    None => {
                        let mut tbl = HashMap::new();
                        tbl.insert(uniq_name.clone(), uniq_method_name.clone());
                        method_env.insert(method.name.as_str(), tbl);
                    }
                };
                let mut uniq_args: Vec<String> = Vec::new();
                let mut env_clone = var_env.clone();
                for arg in method.parameters.iter() {
                    let uniq_arg: String = format!("#{}_{}", arg, method.ann);
                    env_clone.push((&arg, uniq_arg.clone()));
                    uniq_args.push(uniq_arg);
                }
                uniq_methods.push(FunDecl {
                    name: uniq_method_name,
                    parameters: uniq_args,
                    body: uniquify(
                        &method.body,
                        env_clone.clone(),
                        class_env.clone(),
                        method_env.clone(),
                    ),
                    ann: (),
                });
            }
            Exp::ClassDef {
                name: uniq_name,
                fields: uniq_fields,
                methods: uniq_methods,
                body: Box::new(uniquify(&body, var_env_clone, class_env, method_env)),
                ann: (),
            }
        }
        Exp::Object {
            class,
            fields,
            ann: _,
        } => {
            let uniq_class: String = match get(&class_env, &class) {
                Some(c) => c,
                None => panic!("Class is guaranteed to be in scope. Uniquify."),
            };
            let uniq_fields = fields
                .iter()
                .map(|field: &Exp<u32>| {
                    uniquify(
                        field,
                        var_env.clone(),
                        class_env.clone(),
                        method_env.clone(),
                    )
                })
                .collect();
            Exp::Object {
                class: uniq_class,
                fields: uniq_fields,
                ann: (),
            }
        }
        Exp::CallMethod {
            object,
            method,
            args,
            ann: _,
        } => {
            let uniqmethod = match method_env.get(method.as_str()) {
                Some(tbl) => tbl,
                None => {
                    panic!("Method is guaranteed to be in scope")
                }
            };
            Exp::CallUniqMethod {
                object: Box::new(uniquify(
                    object,
                    var_env.clone(),
                    class_env.clone(),
                    method_env.clone(),
                )),
                uniqmethod: uniqmethod.clone(),
                args: args
                    .iter()
                    .map(|arg: &Exp<u32>| {
                        uniquify(arg, var_env.clone(), class_env.clone(), method_env.clone())
                    })
                    .collect(),
                ann: (),
            }
        }
        Exp::SetField {
            field,
            value,
            ann: _,
        } => {
            let uniq_field: String = match get(&var_env, &field) {
                Some(x) => x,
                None => panic!("Field is guaranteed to be in scope"),
            };
            Exp::SetField {
                field: uniq_field,
                value: Box::new(uniquify(
                    value,
                    var_env.clone(),
                    class_env.clone(),
                    method_env.clone(),
                )),
                ann: (),
            }
        }
        Exp::MakeClosure { .. } | Exp::CallUniqMethod { .. } | Exp::MethodDefs { .. } => {
            panic!("Should never exist during uniquify!")
        }
    }
}

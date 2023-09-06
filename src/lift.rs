use crate::scope::get;
use crate::syntax::{
    ClassInfo, Exp, FunDecl, MethodDecl, Prim2, SurfFunDecl, SurfMethodDecl, SurfProg,
};

use std::collections::HashMap;

pub fn class_lift<'exp, Ann>(
    p: &'exp Exp<Ann>,
    env: Vec<(&'exp str, (String, usize))>,
    acc_class: HashMap<String, ClassInfo>,
) -> (HashMap<String, ClassInfo>, SurfProg<()>) {
    match p {
        Exp::Num(i, _) => (acc_class.clone(), Exp::Num(*i, ())),
        Exp::Bool(b, _) => (acc_class.clone(), Exp::Bool(*b, ())),
        Exp::Var(x, _) => match get(&env, x) {
            Some((class_array, idx)) => (
                acc_class.clone(),
                Exp::Prim2(
                    Prim2::ArrayGet,
                    Box::new(Exp::Var(class_array.clone(), ())),
                    Box::new(Exp::Num(idx as i64, ())),
                    (),
                ),
            ),
            None => (acc_class.clone(), Exp::Var(x.clone(), ())),
        },
        Exp::Prim1(prim, p, _) => {
            let (class_map, main) = class_lift(p, env.clone(), acc_class.clone());
            (class_map, Exp::Prim1(*prim, Box::new(main), ()))
        }
        Exp::Prim2(prim, p1, p2, _) => {
            let class_map: HashMap<String, ClassInfo> = acc_class.clone();
            let (class_map, main1) = class_lift(p1, env.clone(), class_map.clone());
            let (class_map, main2) = class_lift(p2, env.clone(), class_map.clone());
            (
                class_map,
                Exp::Prim2(*prim, Box::new(main1), Box::new(main2), ()),
            )
        }
        Exp::Let {
            bindings,
            body,
            ann: _,
        } => {
            let mut class_map: HashMap<String, ClassInfo> = acc_class.clone();
            let mut main_bindings: Vec<(String, Exp<()>)> = Vec::new();
            for (x, p) in bindings.iter() {
                let (class_map_binding, main_binding) = class_lift(p, env.clone(), class_map);
                class_map = class_map_binding;
                main_bindings.push((x.clone(), main_binding));
            }
            let (class_map, main_body) = class_lift(body, env.clone(), class_map);
            (
                class_map,
                Exp::Let {
                    bindings: main_bindings,
                    body: Box::new(main_body),
                    ann: (),
                },
            )
        }
        Exp::If {
            cond,
            thn,
            els,
            ann: _,
        } => {
            let class_map: HashMap<String, ClassInfo> = acc_class.clone();
            let (class_map, main_cond) = class_lift(cond, env.clone(), class_map.clone());
            let (class_map, main_thn) = class_lift(thn, env.clone(), class_map.clone());
            let (class_map, main_els) = class_lift(els, env.clone(), class_map.clone());
            (
                class_map,
                Exp::If {
                    cond: Box::new(main_cond),
                    thn: Box::new(main_thn),
                    els: Box::new(main_els),
                    ann: (),
                },
            )
        }
        Exp::Array(array, _) => {
            let mut class_map: HashMap<String, ClassInfo> = acc_class.clone();
            let mut main_array = Vec::new();
            for element in array.iter() {
                let (class_map_element, main_element) =
                    class_lift(element, env.clone(), class_map.clone());
                main_array.push(main_element);
                class_map = class_map_element;
            }
            (class_map, Exp::Array(main_array, ()))
        }
        Exp::ArraySet {
            array,
            index,
            new_value,
            ann: _,
        } => {
            let class_map: HashMap<String, ClassInfo> = acc_class.clone();
            let (class_map, main_array) = class_lift(array, env.clone(), class_map.clone());
            let (class_map, main_index) = class_lift(index, env.clone(), class_map.clone());
            let (class_map, main_value) = class_lift(new_value, env.clone(), class_map.clone());
            (
                class_map,
                Exp::ArraySet {
                    array: Box::new(main_array),
                    index: Box::new(main_index),
                    new_value: Box::new(main_value),
                    ann: (),
                },
            )
        }
        Exp::Semicolon { e1, e2, ann: _ } => {
            let class_map: HashMap<String, ClassInfo> = acc_class.clone();
            let (class_map, main1) = class_lift(e1, env.clone(), class_map.clone());
            let (class_map, main2) = class_lift(e2, env.clone(), class_map.clone());
            (
                class_map,
                Exp::Semicolon {
                    e1: Box::new(main1),
                    e2: Box::new(main2),
                    ann: (),
                },
            )
        }
        Exp::FunDefs {
            decls,
            body,
            ann: _,
        } => {
            let mut class_map: HashMap<String, ClassInfo> = acc_class.clone();
            let mut main_decls = Vec::new();
            for decl in decls.iter() {
                let (class_map_body, main_body) =
                    class_lift(&decl.body, env.clone(), class_map.clone());
                main_decls.push(FunDecl {
                    name: decl.name.clone(),
                    parameters: decl.parameters.clone(),
                    body: main_body,
                    ann: (),
                });
                class_map = class_map_body;
            }
            let (class_map, main_body) = class_lift(body, env.clone(), class_map.clone());
            (
                class_map,
                Exp::FunDefs {
                    decls: main_decls,
                    body: Box::new(main_body),
                    ann: (),
                },
            )
        }
        Exp::Call(fun, args, _) => {
            let mut class_map: HashMap<String, ClassInfo> = acc_class.clone();
            let mut main_args = Vec::new();
            for arg in args.iter() {
                let (class_map_arg, main_arg) = class_lift(arg, env.clone(), class_map.clone());
                main_args.push(main_arg);
                class_map = class_map_arg;
            }
            let (class_map, main_fun) = class_lift(fun, env.clone(), class_map.clone());
            (class_map, Exp::Call(Box::new(main_fun), main_args, ()))
        }
        Exp::Object {
            class,
            fields,
            ann: _,
        } => {
            let mut class_map: HashMap<String, ClassInfo> = acc_class.clone();
            let mut main_fields = Vec::new();
            for field in fields.iter() {
                let (class_map_field, main_field) =
                    class_lift(field, env.clone(), class_map.clone());
                main_fields.push(main_field);
                class_map = class_map_field;
            }
            (
                class_map,
                Exp::Object {
                    class: class.clone(),
                    fields: main_fields,
                    ann: (),
                },
            )
        }
        Exp::CallUniqMethod {
            object,
            uniqmethod,
            args,
            ann: _,
        } => {
            let mut class_map: HashMap<String, ClassInfo> = acc_class.clone();
            let mut main_args = Vec::new();
            for arg in args.iter() {
                let (class_map_arg, main_arg) = class_lift(arg, env.clone(), class_map.clone());
                main_args.push(main_arg);
                class_map = class_map_arg;
            }
            let (class_map, main_object) = class_lift(object, env.clone(), class_map.clone());
            (
                class_map,
                Exp::CallUniqMethod {
                    object: Box::new(main_object),
                    uniqmethod: uniqmethod.clone(),
                    args: main_args,
                    ann: (),
                },
            )
        }
        Exp::SetField {
            field,
            value,
            ann: _,
        } => {
            let class_map: HashMap<String, ClassInfo> = acc_class.clone();
            let (array, idx) = match get(&env, field) {
                Some((class_array, idx)) => (class_array, idx),
                None => panic!("Trying to set variable as class field"),
            };
            let (class_map, main_value) = class_lift(value, env.clone(), class_map.clone());
            (
                class_map,
                Exp::ArraySet {
                    array: Box::new(Exp::Var(array.clone(), ())),
                    index: Box::new(Exp::Num(idx as i64, ())),
                    new_value: Box::new(main_value),
                    ann: (),
                },
            )
        }
        Exp::ClassDef {
            name,
            fields,
            methods,
            body,
            ann: _,
        } => {
            let class_map: HashMap<String, ClassInfo> = acc_class.clone();
            let (mut class_map, main_body) = class_lift(body, env.clone(), class_map.clone());
            let classid = class_map.len() + 1;
            class_map.insert(
                name.clone(),
                ClassInfo {
                    id: classid,
                    fieldsize: fields.len(),
                },
            );
            let mut method_env = env.clone();
            let class_array = format!("#{}_array", name);
            for (idx, field) in fields.iter().enumerate() {
                method_env.push((&field, (class_array.clone(), idx)));
            }
            let mut main_methods = Vec::new();
            for method in methods.iter() {
                let mut extended_params = vec![class_array.clone()];
                extended_params.extend(method.parameters.clone());
                let (class_map_method, main_method_body) =
                    class_lift(&method.body, method_env.clone(), class_map.clone());
                class_map = class_map_method;
                main_methods.push(FunDecl {
                    name: method.name.clone(),
                    parameters: extended_params.clone(),
                    body: main_method_body.clone(),
                    ann: (),
                });
            }
            (
                class_map,
                Exp::MethodDefs {
                    class: classid,
                    decls: main_methods.clone(),
                    body: Box::new(main_body),
                    ann: (),
                },
            )
        }
        Exp::CallMethod { .. }
        | Exp::MethodDefs { .. }
        | Exp::Lambda { .. }
        | Exp::MakeClosure { .. } => {
            panic!("Should never exist during lambda lift!")
        }
    }
}

pub fn lambda_lift<'exp, Ann>(
    p: &'exp Exp<Ann>,
    mut env: Vec<(&'exp str, ())>,
) -> (Vec<SurfFunDecl<()>>, Vec<SurfMethodDecl<()>>, SurfProg<()>) {
    match p {
        Exp::Num(i, _) => (Vec::new(), Vec::new(), Exp::Num(*i, ())),
        Exp::Bool(b, _) => (Vec::new(), Vec::new(), Exp::Bool(*b, ())),
        Exp::Var(x, _) => (Vec::new(), Vec::new(), Exp::Var(x.clone(), ())),
        Exp::Prim1(prim, p, _) => {
            let (fun_vec, method_vec, main) = lambda_lift(&p, env.clone());
            (fun_vec, method_vec, Exp::Prim1(*prim, Box::new(main), ()))
        }
        Exp::Prim2(prim, p1, p2, _) => {
            let mut fun_vec: Vec<FunDecl<Exp<()>, ()>> = Vec::new();
            let mut method_vec: Vec<MethodDecl<Exp<()>, ()>> = Vec::new();
            let (fun_vec1, method_vec1, main1) = lambda_lift(&p1, env.clone());
            let (fun_vec2, method_vec2, main2) = lambda_lift(&p2, env.clone());
            fun_vec.extend(fun_vec1);
            fun_vec.extend(fun_vec2);
            method_vec.extend(method_vec1);
            method_vec.extend(method_vec2);
            (
                fun_vec,
                method_vec,
                Exp::Prim2(*prim, Box::new(main1), Box::new(main2), ()),
            )
        }
        Exp::Let {
            bindings,
            body,
            ann: _,
        } => {
            let mut fun_vec: Vec<FunDecl<Exp<()>, ()>> = Vec::new();
            let mut method_vec: Vec<MethodDecl<Exp<()>, ()>> = Vec::new();
            let mut main_bindings: Vec<(String, Exp<()>)> = Vec::new();
            for (x, p) in bindings.iter() {
                let (fun_vec_binding, method_vec_binding, main_binding) =
                    lambda_lift(&p, env.clone());
                fun_vec.extend(fun_vec_binding);
                method_vec.extend(method_vec_binding);
                env.push((&x, ()));
                main_bindings.push((x.clone(), main_binding));
            }
            let (fun_vec_body, method_vec_body, main_body) = lambda_lift(&body, env.clone());
            fun_vec.extend(fun_vec_body);
            method_vec.extend(method_vec_body);
            (
                fun_vec,
                method_vec,
                Exp::Let {
                    bindings: main_bindings,
                    body: Box::new(main_body),
                    ann: (),
                },
            )
        }
        Exp::If {
            cond,
            thn,
            els,
            ann: _,
        } => {
            let mut fun_vec: Vec<FunDecl<Exp<()>, ()>> = Vec::new();
            let mut method_vec: Vec<MethodDecl<Exp<()>, ()>> = Vec::new();
            let (fun_vec_cond, method_vec_cond, main_cond) = lambda_lift(&cond, env.clone());
            let (fun_vec_thn, method_vec_thn, main_thn) = lambda_lift(&thn, env.clone());
            let (fun_vec_els, method_vec_els, main_els) = lambda_lift(&els, env.clone());
            fun_vec.extend(fun_vec_cond);
            fun_vec.extend(fun_vec_thn);
            fun_vec.extend(fun_vec_els);
            method_vec.extend(method_vec_cond);
            method_vec.extend(method_vec_thn);
            method_vec.extend(method_vec_els);
            (
                fun_vec,
                method_vec,
                Exp::If {
                    cond: Box::new(main_cond),
                    thn: Box::new(main_thn),
                    els: Box::new(main_els),
                    ann: (),
                },
            )
        }
        Exp::Array(array, _) => {
            let mut fun_vec: Vec<FunDecl<Exp<()>, ()>> = Vec::new();
            let mut method_vec: Vec<MethodDecl<Exp<()>, ()>> = Vec::new();
            let mut main_array: Vec<Exp<()>> = Vec::new();
            for element in array.iter() {
                let (fun_vec_element, method_vec_element, main_element) =
                    lambda_lift(&element, env.clone());
                fun_vec.extend(fun_vec_element);
                method_vec.extend(method_vec_element);
                main_array.push(main_element);
            }
            (fun_vec, method_vec, Exp::Array(main_array, ()))
        }
        Exp::ArraySet {
            array,
            index,
            new_value,
            ann: _,
        } => {
            let mut fun_vec: Vec<FunDecl<Exp<()>, ()>> = Vec::new();
            let mut method_vec: Vec<MethodDecl<Exp<()>, ()>> = Vec::new();
            let (fun_vec_array, method_vec_array, main_array) = lambda_lift(&array, env.clone());
            let (fun_vec_index, method_vec_index, main_index) = lambda_lift(&index, env.clone());
            let (fun_vec_new_value, method_vec_new_value, main_new_value) =
                lambda_lift(&new_value, env.clone());
            fun_vec.extend(fun_vec_array);
            fun_vec.extend(fun_vec_index);
            fun_vec.extend(fun_vec_new_value);
            method_vec.extend(method_vec_array);
            method_vec.extend(method_vec_index);
            method_vec.extend(method_vec_new_value);
            (
                fun_vec,
                method_vec,
                Exp::ArraySet {
                    array: Box::new(main_array),
                    index: Box::new(main_index),
                    new_value: Box::new(main_new_value),
                    ann: (),
                },
            )
        }
        Exp::Semicolon { e1, e2, ann: _ } => {
            let mut fun_vec: Vec<FunDecl<Exp<()>, ()>> = Vec::new();
            let mut method_vec: Vec<MethodDecl<Exp<()>, ()>> = Vec::new();
            let (fun_vec1, method_vec1, main1) = lambda_lift(&e1, env.clone());
            let (fun_vec2, method_vec2, main2) = lambda_lift(&e2, env.clone());
            fun_vec.extend(fun_vec1);
            fun_vec.extend(fun_vec2);
            method_vec.extend(method_vec1);
            method_vec.extend(method_vec2);
            (
                fun_vec,
                method_vec,
                Exp::Semicolon {
                    e1: Box::new(main1),
                    e2: Box::new(main2),
                    ann: (),
                },
            )
        }
        Exp::FunDefs {
            decls,
            body,
            ann: _,
        } => {
            // Push functions into the environment
            let mut env_varname: String = String::from("env");
            for FunDecl { name, .. } in decls.iter() {
                env.push((name, ()));
                env_varname = format!("{}_{}", env_varname, name);
            }
            let global: Vec<Exp<()>> = env
                .iter()
                .map(|(x, _)| Exp::Var(x.to_string(), ()))
                .collect();
            let mut fun_vec: Vec<FunDecl<Exp<()>, ()>> = Vec::new();
            let mut method_vec: Vec<MethodDecl<Exp<()>, ()>> = Vec::new();
            for FunDecl {
                name,
                parameters,
                body,
                ann: _,
            } in decls.iter()
            {
                // Construct function environment
                let mut env_clone: Vec<(&str, ())> = env.clone();
                env_clone.push((&env_varname, ()));
                for arg in parameters.iter() {
                    env_clone.push((&arg, ()));
                }
                // Add array "env" to parameters
                let mut main_parameters: Vec<String> = vec![env_varname.clone()];
                main_parameters.extend(parameters.clone());
                // Get the lambda lifted function body
                let (fun_vec_body, method_vec_body, fun_body) = lambda_lift(&body, env_clone);
                fun_vec.extend(fun_vec_body);
                method_vec.extend(method_vec_body);
                // Add let bindings to the function body according to the environment
                let main_bindings: Vec<(String, Exp<()>)> = env
                    .iter()
                    .enumerate()
                    .map(|(i, (x, _))| {
                        (
                            x.to_string().clone(),
                            Exp::Prim2(
                                Prim2::ArrayGet,
                                Box::new(Exp::Var(env_varname.clone(), ())),
                                Box::new(Exp::Num(i as i64, ())),
                                (),
                            ),
                        )
                    })
                    .collect();
                let main_body: Exp<()> = Exp::Let {
                    bindings: main_bindings,
                    body: Box::new(fun_body),
                    ann: (),
                };
                // Push the function to fun_vec
                fun_vec.push(FunDecl {
                    name: name.clone(),
                    parameters: main_parameters.clone(),
                    body: main_body.clone(),
                    ann: (),
                });
            }
            // Get the lambda lifted body
            let (fun_vec_body, method_vec_body, main_body) = lambda_lift(&body, env.clone());
            fun_vec.extend(fun_vec_body);
            method_vec.extend(method_vec_body);
            // Construct the environment capture array
            let env_closure: Vec<Exp<()>> = global
                .iter()
                .enumerate()
                .map(|(i, x)| {
                    if i < global.len() - decls.len() {
                        x.clone()
                    } else {
                        Exp::Num(0, ())
                    }
                })
                .collect();
            // Make closures
            let mut closure_bindings: Vec<(String, Exp<()>)> =
                vec![(env_varname.clone(), Exp::Array(env_closure, ()))];
            for decl in decls.iter() {
                closure_bindings.push((
                    decl.name.clone(),
                    Exp::MakeClosure {
                        arity: decl.parameters.len(),
                        label: decl.name.clone(),
                        env: Box::new(Exp::Var(env_varname.clone(), ())),
                        ann: (),
                    },
                ));
            }
            // Modify the environment capture array to include the functions
            let main_body: Exp<()> =
                decls
                    .iter()
                    .rev()
                    .enumerate()
                    .fold(main_body, |acc, (i, decl)| Exp::Semicolon {
                        e1: Box::new(Exp::ArraySet {
                            array: Box::new(Exp::Var(env_varname.clone(), ())),
                            index: Box::new(Exp::Num((global.len() - i - 1) as i64, ())),
                            new_value: Box::new(Exp::Var(decl.name.clone(), ())),
                            ann: (),
                        }),
                        e2: Box::new(acc),
                        ann: (),
                    });
            (
                fun_vec,
                method_vec,
                Exp::Let {
                    bindings: closure_bindings,
                    body: Box::new(main_body),
                    ann: (),
                },
            )
        }
        Exp::Call(fun, args, _) => {
            let mut fun_vec: Vec<FunDecl<Exp<()>, ()>> = Vec::new();
            let mut method_vec: Vec<MethodDecl<Exp<()>, ()>> = Vec::new();
            let (fun_vec_fun, method_vec_fun, main_fun) = lambda_lift(fun, env.clone());
            fun_vec.extend(fun_vec_fun);
            method_vec.extend(method_vec_fun);
            let mut main_args: Vec<Exp<()>> = Vec::new();
            for arg in args.iter() {
                let (fun_vec_arg, method_vec_arg, main_arg) = lambda_lift(&arg, env.clone());
                fun_vec.extend(fun_vec_arg);
                method_vec.extend(method_vec_arg);
                main_args.push(main_arg);
            }
            (
                fun_vec,
                method_vec,
                Exp::Call(Box::new(main_fun), main_args, ()),
            )
        }
        Exp::Object {
            class,
            fields,
            ann: _,
        } => {
            let mut fun_vec: Vec<FunDecl<Exp<()>, ()>> = Vec::new();
            let mut method_vec: Vec<MethodDecl<Exp<()>, ()>> = Vec::new();
            let mut main_fields: Vec<Exp<()>> = Vec::new();
            for field in fields.iter() {
                let (fun_vec_field, method_vec_field, main_field) =
                    lambda_lift(&field, env.clone());
                fun_vec.extend(fun_vec_field);
                method_vec.extend(method_vec_field);
                main_fields.push(main_field);
            }
            (
                fun_vec,
                method_vec,
                Exp::Object {
                    class: class.clone(),
                    fields: main_fields,
                    ann: (),
                },
            )
        }
        Exp::CallUniqMethod {
            object,
            uniqmethod,
            args,
            ann: _,
        } => {
            let mut fun_vec: Vec<FunDecl<Exp<()>, ()>> = Vec::new();
            let mut method_vec: Vec<MethodDecl<Exp<()>, ()>> = Vec::new();
            let mut main_args: Vec<Exp<()>> = Vec::new();
            for arg in args.iter() {
                let (fun_vec_arg, method_vec_arg, main_arg) = lambda_lift(&arg, env.clone());
                fun_vec.extend(fun_vec_arg);
                method_vec.extend(method_vec_arg);
                main_args.push(main_arg);
            }
            let (fun_vec_object, method_vec_object, main_object) = lambda_lift(object, env.clone());
            fun_vec.extend(fun_vec_object);
            method_vec.extend(method_vec_object);
            (
                fun_vec,
                method_vec,
                Exp::CallUniqMethod {
                    object: Box::new(main_object),
                    uniqmethod: uniqmethod.clone(),
                    args: main_args.clone(),
                    ann: (),
                },
            )
        }
        Exp::MethodDefs {
            class,
            decls,
            body,
            ann: _,
        } => {
            // Push functions into the environment
            let mut env_varname: String = String::from("env");
            for FunDecl { name, .. } in decls.iter() {
                env.push((name, ()));
                env_varname = format!("{}_{}", env_varname, name);
            }
            let global: Vec<Exp<()>> = env
                .iter()
                .map(|(x, _)| Exp::Var(x.to_string(), ()))
                .collect();
            let mut fun_vec: Vec<FunDecl<Exp<()>, ()>> = Vec::new();
            let mut method_vec: Vec<MethodDecl<Exp<()>, ()>> = Vec::new();
            for FunDecl {
                name,
                parameters,
                body,
                ann: _,
            } in decls.iter()
            {
                // Construct function environment
                let mut env_clone: Vec<(&str, ())> = env.clone();
                env_clone.push((&env_varname, ()));
                for arg in parameters.iter() {
                    env_clone.push((&arg, ()));
                }
                // Add array "env" to parameters
                let mut main_parameters: Vec<String> = vec![env_varname.clone()];
                main_parameters.extend(parameters.clone());
                // Get the lambda lifted function body
                let (fun_vec_body, method_vec_body, fun_body) = lambda_lift(&body, env_clone);
                fun_vec.extend(fun_vec_body);
                method_vec.extend(method_vec_body);
                // Add let bindings to the function body according to the environment
                let main_bindings: Vec<(String, Exp<()>)> = env
                    .iter()
                    .enumerate()
                    .map(|(i, (x, _))| {
                        (
                            x.to_string().clone(),
                            Exp::Prim2(
                                Prim2::ArrayGet,
                                Box::new(Exp::Var(env_varname.clone(), ())),
                                Box::new(Exp::Num(i as i64, ())),
                                (),
                            ),
                        )
                    })
                    .collect();
                let main_body: Exp<()> = Exp::Let {
                    bindings: main_bindings,
                    body: Box::new(fun_body),
                    ann: (),
                };
                // Push the function to fun_vec
                method_vec.push(MethodDecl {
                    class: *class,
                    fundecl: FunDecl {
                        name: name.clone(),
                        parameters: main_parameters.clone(),
                        body: main_body.clone(),
                        ann: (),
                    },
                });
            }
            // Get the lambda lifted body
            let (fun_vec_body, method_vec_body, main_body) = lambda_lift(&body, env.clone());
            fun_vec.extend(fun_vec_body);
            method_vec.extend(method_vec_body);
            // Construct the environment capture array
            let env_closure: Vec<Exp<()>> = global
                .iter()
                .enumerate()
                .map(|(i, x)| {
                    if i < global.len() - decls.len() {
                        x.clone()
                    } else {
                        Exp::Num(0, ())
                    }
                })
                .collect();
            // Make closures
            let mut closure_bindings: Vec<(String, Exp<()>)> =
                vec![(env_varname.clone(), Exp::Array(env_closure, ()))];
            for decl in decls.iter() {
                closure_bindings.push((
                    decl.name.clone(),
                    Exp::MakeClosure {
                        arity: decl.parameters.len(),
                        label: decl.name.clone(),
                        env: Box::new(Exp::Var(env_varname.clone(), ())),
                        ann: (),
                    },
                ));
            }
            // Modify the environment capture array to include the functions
            let main_body: Exp<()> =
                decls
                    .iter()
                    .rev()
                    .enumerate()
                    .fold(main_body, |acc, (i, decl)| Exp::Semicolon {
                        e1: Box::new(Exp::ArraySet {
                            array: Box::new(Exp::Var(env_varname.clone(), ())),
                            index: Box::new(Exp::Num((global.len() - i - 1) as i64, ())),
                            new_value: Box::new(Exp::Var(decl.name.clone(), ())),
                            ann: (),
                        }),
                        e2: Box::new(acc),
                        ann: (),
                    });
            (
                fun_vec,
                method_vec,
                Exp::Let {
                    bindings: closure_bindings,
                    body: Box::new(main_body),
                    ann: (),
                },
            )
        }
        Exp::Lambda { .. }
        | Exp::ClassDef { .. }
        | Exp::SetField { .. }
        | Exp::CallMethod { .. }
        | Exp::MakeClosure { .. } => {
            panic!("Should never exist during lambda lift!")
        }
    }
}

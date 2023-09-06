use std::collections::HashMap;

use crate::syntax::{Exp, ImmExp, Prim1, Prim2, SeqExp};

fn simple_exp_to_imm(e: &Exp<u32>) -> Option<ImmExp> {
    match e {
        Exp::Num(i, _) => Some(ImmExp::Num(*i)),
        Exp::Bool(b, _) => Some(ImmExp::Bool(*b)),
        Exp::Var(x, _) => Some(ImmExp::Var(x.clone())),
        _ => None,
    }
}

fn sequentialize_prim1_help(prim: Prim1, e: &Exp<u32>, ann: u32) -> SeqExp<()> {
    match simple_exp_to_imm(&e) {
        Some(imm) => SeqExp::Prim1(prim, imm, ()),
        None => {
            let x = format!("#prim1_{}", ann);
            SeqExp::Let {
                var: x.clone(),
                bound_exp: Box::new(sequentialize(&e)),
                body: Box::new(SeqExp::Prim1(prim, ImmExp::Var(x.clone()), ())),
                ann: (),
            }
        }
    }
}

fn sequentialize_prim2_help(prim: Prim2, e1: &Exp<u32>, e2: &Exp<u32>, ann: u32) -> SeqExp<()> {
    match (simple_exp_to_imm(&e1), simple_exp_to_imm(&e2)) {
        (Some(imm1), Some(imm2)) => SeqExp::Prim2(prim, imm1, imm2, ()),
        (None, Some(imm)) => {
            let x = format!("#prim2_1_{}", ann);
            SeqExp::Let {
                var: x.clone(),
                bound_exp: Box::new(sequentialize(&e1)),
                body: Box::new(SeqExp::Prim2(prim, ImmExp::Var(x.clone()), imm, ())),
                ann: (),
            }
        }
        (Some(imm), None) => {
            let x = format!("#prim2_2_{}", ann);
            SeqExp::Let {
                var: x.clone(),
                bound_exp: Box::new(sequentialize(&e2)),
                body: Box::new(SeqExp::Prim2(prim, imm, ImmExp::Var(x.clone()), ())),
                ann: (),
            }
        }
        (None, None) => {
            let x1 = format!("#prim2_1_{}", ann);
            let x2 = format!("#prim2_2_{}", ann);
            SeqExp::Let {
                var: x1.clone(),
                bound_exp: Box::new(sequentialize(&e1)),
                body: Box::new(SeqExp::Let {
                    var: x2.clone(),
                    bound_exp: Box::new(sequentialize(&e2)),
                    body: Box::new(SeqExp::Prim2(
                        prim,
                        ImmExp::Var(x1.clone()),
                        ImmExp::Var(x2.clone()),
                        (),
                    )),
                    ann: (),
                }),
                ann: (),
            }
        }
    }
}

fn sequentialize_let_help(bindings: &[(String, Exp<u32>)], body: &Exp<u32>) -> SeqExp<()> {
    bindings
        .iter()
        .rev()
        .fold(sequentialize(&body), |acc, (x, def)| SeqExp::Let {
            var: x.clone(),
            bound_exp: Box::new(sequentialize(&def)),
            body: Box::new(acc),
            ann: (),
        })
}

fn sequentialize_if_help(cond: &Exp<u32>, thn: &Exp<u32>, els: &Exp<u32>, ann: u32) -> SeqExp<()> {
    match simple_exp_to_imm(&cond) {
        Some(imm) => SeqExp::If {
            cond: imm,
            thn: Box::new(sequentialize(&thn)),
            els: Box::new(sequentialize(&els)),
            ann: (),
        },
        None => {
            let x = format!("#if_{}", ann);
            SeqExp::Let {
                var: x.clone(),
                bound_exp: Box::new(sequentialize(&cond)),
                body: Box::new(SeqExp::If {
                    cond: ImmExp::Var(x.clone()),
                    thn: Box::new(sequentialize(&thn)),
                    els: Box::new(sequentialize(&els)),
                    ann: (),
                }),
                ann: (),
            }
        }
    }
}

fn sequentialize_array_help(array: &Vec<Exp<u32>>, ann: u32) -> SeqExp<()> {
    let mut seq_array: Vec<ImmExp> = Vec::new();
    let mut bindings: Vec<(String, Exp<u32>)> = Vec::new();
    for (i, element) in array.iter().enumerate() {
        match simple_exp_to_imm(element) {
            Some(imm) => seq_array.push(imm),
            None => {
                let x: String = format!("#array_{}_element_{}", ann, i);
                bindings.push((x.clone(), element.clone()));
                seq_array.push(ImmExp::Var(x.clone()));
            }
        };
    }
    bindings
        .iter()
        .rev()
        .fold(SeqExp::Array(seq_array, ()), |acc, (x, def)| SeqExp::Let {
            var: x.clone(),
            bound_exp: Box::new(sequentialize(&def)),
            body: Box::new(acc),
            ann: (),
        })
}

fn sequentialize_arrayset_help(
    array: &Exp<u32>,
    index: &Exp<u32>,
    new_value: &Exp<u32>,
    ann: u32,
) -> SeqExp<()> {
    let mut bindings: Vec<(String, Exp<u32>)> = Vec::new();
    let imm_array: ImmExp = match simple_exp_to_imm(&array) {
        Some(imm) => imm,
        None => {
            let x: String = format!("#arrayset_array_{}", ann);
            bindings.push((x.clone(), array.clone()));
            ImmExp::Var(x.clone())
        }
    };
    let imm_index: ImmExp = match simple_exp_to_imm(&index) {
        Some(imm) => imm,
        None => {
            let x: String = format!("#arrayset_index_{}", ann);
            bindings.push((x.clone(), index.clone()));
            ImmExp::Var(x.clone())
        }
    };
    let imm_new_value: ImmExp = match simple_exp_to_imm(&new_value) {
        Some(imm) => imm,
        None => {
            let x: String = format!("#arrayset_new_value_{}", ann);
            bindings.push((x.clone(), new_value.clone()));
            ImmExp::Var(x.clone())
        }
    };
    bindings.iter().rev().fold(
        SeqExp::ArraySet {
            array: imm_array,
            index: imm_index,
            new_value: imm_new_value,
            ann: (),
        },
        |acc, (x, def)| SeqExp::Let {
            var: x.clone(),
            bound_exp: Box::new(sequentialize(&def)),
            body: Box::new(acc),
            ann: (),
        },
    )
}

fn sequentialize_semicolon_help(e1: &Exp<u32>, e2: &Exp<u32>, ann: u32) -> SeqExp<()> {
    SeqExp::Let {
        var: format!("#dummy_{}", ann),
        bound_exp: Box::new(sequentialize(&e1)),
        body: Box::new(sequentialize(&e2)),
        ann: (),
    }
}

fn sequentialize_call_help(fun: &Exp<u32>, args: &Vec<Exp<u32>>, ann: u32) -> SeqExp<()> {
    let mut seq_args: Vec<ImmExp> = Vec::new();
    let mut bindings: Vec<(String, Exp<u32>)> = Vec::new();
    let imm_fun: ImmExp = match simple_exp_to_imm(&fun) {
        Some(imm) => imm,
        None => {
            let x: String = format!("#function_{}", ann);
            bindings.push((x.clone(), fun.clone()));
            ImmExp::Var(x.clone())
        }
    };
    for (i, arg) in args.iter().enumerate() {
        match simple_exp_to_imm(arg) {
            Some(imm) => seq_args.push(imm),
            None => {
                let x: String = format!("#call_function_{}_arg_{}", ann, i);
                bindings.push((x.clone(), arg.clone()));
                seq_args.push(ImmExp::Var(x.clone()));
            }
        };
    }
    bindings.iter().rev().fold(
        SeqExp::CallClosure {
            fun: imm_fun,
            args: seq_args,
            ann: (),
        },
        |acc, (x, def)| SeqExp::Let {
            var: x.clone(),
            bound_exp: Box::new(sequentialize(&def)),
            body: Box::new(acc),
            ann: (),
        },
    )
}

fn sequentialize_makeclosure_help(arity: usize, label: String, env: &Exp<u32>) -> SeqExp<()> {
    match simple_exp_to_imm(&env) {
        Some(imm) => SeqExp::MakeClosure {
            arity: arity,
            label: label,
            env: imm,
            ann: (),
        },
        None => {
            panic!("Env is guaranteed to be ImmExp when generating")
        }
    }
}

fn sequentialize_object_help(class: String, fields: Vec<Exp<u32>>, ann: u32) -> SeqExp<()> {
    let mut seq_fields: Vec<ImmExp> = Vec::new();
    let mut bindings: Vec<(String, Exp<u32>)> = Vec::new();
    for (i, field) in fields.iter().enumerate() {
        match simple_exp_to_imm(field) {
            Some(imm) => seq_fields.push(imm),
            None => {
                let x: String = format!("#object_{}_{}_field_{}", class, ann, i);
                bindings.push((x.clone(), field.clone()));
                seq_fields.push(ImmExp::Var(x.clone()));
            }
        };
    }
    bindings.iter().rev().fold(
        SeqExp::Object {
            class: class.clone(),
            fields: seq_fields.clone(),
            ann: (),
        },
        |acc, (x, def)| SeqExp::Let {
            var: x.clone(),
            bound_exp: Box::new(sequentialize(&def)),
            body: Box::new(acc),
            ann: (),
        },
    )
}

fn sequentialize_callmethod_help(
    object: &Exp<u32>,
    method_tbl: HashMap<String, String>,
    args: &Vec<Exp<u32>>,
    ann: u32,
) -> SeqExp<()> {
    let mut seq_args: Vec<ImmExp> = Vec::new();
    let mut bindings: Vec<(String, Exp<u32>)> = Vec::new();
    let imm_object: ImmExp = match simple_exp_to_imm(&object) {
        Some(imm) => imm,
        None => {
            let x: String = format!("#object_{}", ann);
            bindings.push((x.clone(), object.clone()));
            ImmExp::Var(x.clone())
        }
    };
    for (i, arg) in args.iter().enumerate() {
        match simple_exp_to_imm(arg) {
            Some(imm) => seq_args.push(imm),
            None => {
                let x: String = format!("#call_method_{}_arg_{}", ann, i);
                bindings.push((x.clone(), arg.clone()));
                seq_args.push(ImmExp::Var(x.clone()));
            }
        };
    }
    bindings.iter().rev().fold(
        SeqExp::CallMethod {
            object: imm_object,
            method: method_tbl.clone(),
            args: seq_args.clone(),
            ann: (),
        },
        |acc, (x, def)| SeqExp::Let {
            var: x.clone(),
            bound_exp: Box::new(sequentialize(&def)),
            body: Box::new(acc),
            ann: (),
        },
    )
}

pub fn sequentialize(p: &Exp<u32>) -> SeqExp<()> {
    match p {
        Exp::Num(i, _) => SeqExp::Imm(ImmExp::Num(*i), ()),
        Exp::Bool(b, _) => SeqExp::Imm(ImmExp::Bool(*b), ()),
        Exp::Var(x, _) => SeqExp::Imm(ImmExp::Var(x.clone()), ()),
        Exp::Prim1(prim, p, ann) => sequentialize_prim1_help(*prim, &p, *ann),
        Exp::Prim2(prim, p1, p2, ann) => sequentialize_prim2_help(*prim, &p1, &p2, *ann),
        Exp::Let {
            bindings,
            body,
            ann: _,
        } => sequentialize_let_help(&bindings, &body),
        Exp::If {
            cond,
            thn,
            els,
            ann,
        } => sequentialize_if_help(&cond, &thn, &els, *ann),
        Exp::Array(array, ann) => sequentialize_array_help(&array, *ann),
        Exp::ArraySet {
            array,
            index,
            new_value,
            ann,
        } => sequentialize_arrayset_help(&array, &index, &new_value, *ann),
        Exp::Semicolon { e1, e2, ann } => sequentialize_semicolon_help(&e1, &e2, *ann),
        Exp::Call(fun, args, ann) => sequentialize_call_help(&fun, &args, *ann),
        Exp::MakeClosure {
            arity,
            label,
            env,
            ann: _,
        } => sequentialize_makeclosure_help(*arity, label.clone(), &env),
        Exp::Object { class, fields, ann } => {
            sequentialize_object_help(class.clone(), fields.clone(), *ann)
        }
        Exp::CallUniqMethod {
            object,
            uniqmethod,
            args,
            ann,
        } => sequentialize_callmethod_help(object, uniqmethod.clone(), args, *ann),
        Exp::Lambda { .. }
        | Exp::FunDefs { .. }
        | Exp::ClassDef { .. }
        | Exp::SetField { .. }
        | Exp::CallMethod { .. }
        | Exp::MethodDefs { .. } => {
            panic!("Should never exist during lambda_lift!")
        }
    }
}

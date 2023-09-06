use crate::syntax::{Exp, Prim1, Prim2, SurfFunDecl, SurfProg};

use std::collections::HashSet;
use std::convert::TryInto;
use std::fmt;
use std::fmt::Display;
use std::rc::Rc;

// The machine is always either evaluating an expression against the stack (and heap) or returning a value
enum Machine<'exp, Ann> {
    Descending {
        e: &'exp Exp<Ann>,
        stk: Stack<'exp, Ann>,
        env: Env,
    },
    Returning {
        v: SnakeVal,
        stk: Stack<'exp, Ann>,
    },
}

/* Semantic Values */
#[derive(Debug, Clone, Copy)]
pub enum SnakeVal {
    Num(i64), // should fit into 63 bits though
    Bool(bool),
    Array(usize),   // index into the array arena
    Closure(usize), // index into the closure arena
}

impl Display for SnakeVal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SnakeVal::Num(n) => write!(f, "{}", n),
            SnakeVal::Bool(b) => write!(f, "{}", b),
            SnakeVal::Closure { .. } => write!(f, "closure"),
            SnakeVal::Array { .. } => write!(f, "array"),
        }
    }
}

/* Semantic Stacks */
enum Stack<'exp, Ann> {
    Done,
    Prim1(Prim1, Box<Stack<'exp, Ann>>),
    Prim2L(Prim2, Closure<'exp, Ann>, Box<Stack<'exp, Ann>>),
    Prim2R(Prim2, SnakeVal, Box<Stack<'exp, Ann>>),
    If {
        thn: &'exp Exp<Ann>,
        els: &'exp Exp<Ann>,
        env: Env,
        stk: Box<Stack<'exp, Ann>>,
    },
    Let {
        var: &'exp str,
        env: Env,
        bindings: Vec<&'exp (String, Exp<Ann>)>,
        body: &'exp Exp<Ann>,
        stk: Box<Stack<'exp, Ann>>,
    },
    CallFun {
        env: Env,
        args: Vec<&'exp Exp<Ann>>,
        stk: Box<Stack<'exp, Ann>>,
    },
    CallArgs {
        fun: SnakeVal,
        evaled_args: Vec<SnakeVal>,
        env: Env,
        remaining_args: Vec<&'exp Exp<Ann>>,
        stk: Box<Stack<'exp, Ann>>,
    },
    Array {
        evaled_parts: Vec<SnakeVal>,
        remaining_parts: Vec<&'exp Exp<Ann>>,
        env: Env,
        stk: Box<Stack<'exp, Ann>>,
    },
    ArraySet1 {
        index: &'exp Exp<Ann>,
        new_value: &'exp Exp<Ann>,
        env: Env,
        stk: Box<Stack<'exp, Ann>>,
    },
    ArraySet2 {
        array: SnakeVal,
        new_value: Closure<'exp, Ann>,
        stk: Box<Stack<'exp, Ann>>,
    },
    ArraySet3 {
        array: SnakeVal,
        index: SnakeVal,
        stk: Box<Stack<'exp, Ann>>,
    },
    Semicolon {
        next: Closure<'exp, Ann>,
        stk: Box<Stack<'exp, Ann>>,
    },
}

/* The semantic Store consists of arenas for allocating arrays and closures */
struct State<'e, Ann> {
    funs: Funs<'e, Ann>,
    heap: Heap,
}
type Heap = Vec<Vec<SnakeVal>>;

struct SemFun<'e, Ann> {
    parameters: &'e [String],
    closure: Closure<'e, Ann>,
}
type Funs<'e, Ann> = Vec<SemFun<'e, Ann>>;

impl<'e, Ann> State<'e, Ann> {
    fn new() -> Self {
        State {
            funs: vec![],
            heap: vec![],
        }
    }

    fn alloc_fun(&mut self, parameters: &'e [String], body: &'e Exp<Ann>, env: Env) -> usize {
        let i = self.funs.len();
        self.funs.push(SemFun {
            parameters: &parameters,
            closure: Closure {
                exp: body,
                env: env.clone(),
            },
        });
        i
    }

    fn alloc_funs(&mut self, decls: &'e [SurfFunDecl<Ann>], mut env: Env) -> Env {
        // Each of the closures captures the same environment: the
        // current environment extended with all of their names
        // i.e., the env we return.
        let i = self.funs.len();
        for (j, d) in decls.iter().enumerate() {
            env = env.push_local(d.name.clone(), SnakeVal::Closure(i + j));
        }
        for d in decls.iter() {
            self.funs.push(SemFun {
                parameters: &d.parameters,
                closure: Closure {
                    exp: &d.body,
                    env: env.clone(),
                },
            });
        }
        env
    }

    fn alloc_array(&mut self, vs: Vec<SnakeVal>) -> usize {
        // can raise an Out of memory error?
        let ptr = self.heap.len();
        self.heap.push(vs);
        ptr
    }
}

// A reference-counted linked list/the functional programmer's List
#[derive(Debug, Clone)]
enum List<T> {
    Empty,
    Cons(T, Rc<List<T>>),
}

impl<A> std::iter::FromIterator<A> for List<A> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = A>,
    {
        let mut l = List::Empty;
        for t in iter {
            l = List::Cons(t, Rc::new(l));
        }
        l
    }
}

fn get<'l, T>(stk: &'l List<(String, T)>, x: &str) -> Option<&'l T> {
    match stk {
        List::Empty => None,
        List::Cons((y, n), stk) => {
            if x == *y {
                Some(n)
            } else {
                get(stk, x)
            }
        }
    }
}

// An environment is implemented as a ref-counted linked list to
// enable sharing/avoid copying
#[derive(Debug, Clone)]
struct Env(Rc<List<(String, SnakeVal)>>);

impl Env {
    fn new() -> Env {
        Env(Rc::new(List::Empty))
    }

    fn push_local(&self, name: String, v: SnakeVal) -> Env {
        Env(Rc::new(List::Cons((name, v), self.0.clone())))
    }

    fn lookup<'l>(&'l self, x: &str) -> Option<&'l SnakeVal> {
        get(&self.0, x)
    }
}

#[derive(Debug, Clone)]
struct Closure<'exp, Ann> {
    exp: &'exp Exp<Ann>,
    env: Env,
}

#[derive(Debug, Clone)]
pub enum InterpErr {
    ExpectedNum {
        who: String,
        got: String,
        msg: String,
    },
    ExpectedBool {
        who: String,
        got: String,
        msg: String,
    },
    ExpectedFun {
        got: String,
    },
    ExpectedArray {
        msg: String,
        got: SnakeVal,
    },
    ArrayOutOfBounds {},
    Overflow {
        msg: String,
    },
    Write {
        msg: String,
    },
    ArityErr {
        expected_arity: usize,
        num_provided: usize,
    },
}

type Interp<T> = Result<T, InterpErr>;

impl Display for InterpErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InterpErr::Write { msg } => write!(f, "I/O Error when printing: {}", msg),
            InterpErr::ExpectedNum { who, got: v, msg } => {
                write!(f, "{} expected a number, but got {} in {}", who, v, msg)
            }
            InterpErr::ExpectedBool { who, got: v, msg } => {
                write!(f, "{} expected a boolean, but got {} in {}", who, v, msg)
            }
            InterpErr::ExpectedFun { got } => {
                write!(
                    f,
                    "Function application expected a closure, but got {}",
                    got
                )
            }
            InterpErr::ExpectedArray { got, msg } => {
                write!(f, "Expected an array but got {} in {}", got, msg)
            }
            InterpErr::Overflow { msg } => write!(f, "Operation {} overflowed", msg),
            InterpErr::ArrayOutOfBounds {} => write!(f, "Array index out of bounds"),
            InterpErr::ArityErr {
                expected_arity,
                num_provided,
            } => {
                write!(
                    f,
                    "Function expecting {} arguments called with {} arguments",
                    expected_arity, num_provided
                )
            }
        }
    }
}

fn prj_bool(v: SnakeVal, who: &str, msg: &str) -> Interp<bool> {
    match v {
        SnakeVal::Bool(b) => Ok(b),
        _ => Err(InterpErr::ExpectedBool {
            who: String::from(who),
            got: v.to_string(),
            msg: String::from(msg),
        }),
    }
}

fn prj_num(v: SnakeVal, who: &str, msg: &str) -> Interp<i64> {
    match v {
        SnakeVal::Num(n) => Ok(n),
        _ => Err(InterpErr::ExpectedNum {
            who: String::from(who),
            got: v.to_string(),
            msg: String::from(msg),
        }),
    }
}

fn prj_fun(v: SnakeVal) -> Interp<usize> {
    match v {
        SnakeVal::Closure(b) => Ok(b),
        _ => Err(InterpErr::ExpectedFun { got: v.to_string() }),
    }
}

fn prj_array<'heap>(v: SnakeVal, msg: &str, heap: &'heap Heap) -> Interp<&'heap [SnakeVal]> {
    match v {
        SnakeVal::Array(ptr) => {
            let arr = heap
                .get(ptr)
                .expect("internal interp error: invalid heap pointer");
            Ok(arr)
        }
        _ => Err(InterpErr::ExpectedArray {
            got: v,
            msg: msg.to_string(),
        }),
    }
}
fn prj_array_mut<'heap>(
    v: SnakeVal,
    msg: &str,
    heap: &'heap mut Heap,
) -> Interp<&'heap mut [SnakeVal]> {
    match v {
        SnakeVal::Array(ptr) => {
            let arr = heap
                .get_mut(ptr)
                .expect("internal interp error: invalid heap pointer");
            Ok(arr)
        }
        _ => Err(InterpErr::ExpectedArray {
            got: v,
            msg: msg.to_string(),
        }),
    }
}

fn valid_index(n: i64) -> Interp<usize> {
    match TryInto::<usize>::try_into(n) {
        Err(_) => Err(InterpErr::ArrayOutOfBounds {}),
        Ok(ix) => Ok(ix),
    }
}

fn print_snake_val<W>(w: &mut W, v: SnakeVal, h: &Heap) -> Interp<SnakeVal>
where
    W: std::io::Write,
{
    fn fixup_err(e: std::io::Error) -> InterpErr {
        InterpErr::Write { msg: e.to_string() }
    }
    fn print_loop<'e, W>(
        w: &mut W,
        v: &SnakeVal,
        h: &Heap,
        mut parents: HashSet<usize>,
    ) -> Result<(), std::io::Error>
    where
        W: std::io::Write,
    {
        match v {
            SnakeVal::Num(n) => write!(w, "{}", n)?,
            SnakeVal::Bool(b) => write!(w, "{}", b)?,
            SnakeVal::Array(ptr) => {
                if parents.contains(ptr) {
                    write!(w, "<loop>")?
                } else {
                    parents.insert(*ptr);
                    let vs = &h[*ptr];
                    write!(w, "[")?;
                    if !vs.is_empty() {
                        print_loop(w, &vs[0], h, parents.clone())?;
                        for v in &vs[1..] {
                            write!(w, ", ")?;
                            print_loop(w, v, h, parents.clone())?;
                        }
                    }
                    write!(w, "]")?;
                }
            }
            SnakeVal::Closure { .. } => {
                write!(w, "<closure>")?;
            }
        }
        Ok(())
    }

    print_loop(w, &v, h, HashSet::new()).map_err(fixup_err)?;
    writeln!(w).map_err(fixup_err)?;
    Ok(v)
}

fn equal_snake_val(v1: &SnakeVal, v2: &SnakeVal, h: &Heap) -> bool {
    fn eq_loop(
        v1: &SnakeVal,
        v2: &SnakeVal,
        _h: &Heap,
        _assumptions: HashSet<(usize, usize)>,
    ) -> bool {
        match (v1, v2) {
            (SnakeVal::Bool(b1), SnakeVal::Bool(b2)) => b1 == b2,
            (SnakeVal::Num(n1), SnakeVal::Num(n2)) => n1 == n2,
            _ => false,
        }
    }
    eq_loop(v1, v2, h, HashSet::new())
}

fn interpret_prim1<W>(p: &Prim1, w: &mut W, v: SnakeVal, h: &Heap) -> Interp<SnakeVal>
where
    W: std::io::Write,
{
    match p {
        Prim1::Add1 => snake_arith(v, SnakeVal::Num(1), |n1, n2| n1.overflowing_add(n2), "add1"),
        Prim1::Sub1 => snake_arith(v, SnakeVal::Num(1), |n1, n2| n1.overflowing_sub(n2), "sub1"),
        Prim1::Not => Ok(SnakeVal::Bool(!prj_bool(v, "logic", "!")?)),
        Prim1::Print => print_snake_val(w, v, h),
        Prim1::IsBool => match v {
            SnakeVal::Bool(_) => Ok(SnakeVal::Bool(true)),
            _ => Ok(SnakeVal::Bool(false)),
        },
        Prim1::Length => {
            let v = prj_array(v, "length", h)?;
            Ok(SnakeVal::Num(v.len().try_into().unwrap()))
        }
        Prim1::IsNum => match v {
            SnakeVal::Num(_) => Ok(SnakeVal::Bool(true)),
            _ => Ok(SnakeVal::Bool(false)),
        },
        Prim1::IsArray => match v {
            SnakeVal::Array(_) => Ok(SnakeVal::Bool(true)),
            _ => Ok(SnakeVal::Bool(false)),
        },
        Prim1::IsFun => match v {
            SnakeVal::Closure(_) => Ok(SnakeVal::Bool(true)),
            _ => Ok(SnakeVal::Bool(false)),
        },
    }
}

static MAX_INT: i64 = 2i64.pow(62) - 1;
static MIN_INT: i64 = -(2i64.pow(62));
fn out_of_bounds(n: i64) -> bool {
    n > MAX_INT || n < MIN_INT
}

fn snake_arith<F>(v1: SnakeVal, v2: SnakeVal, arith: F, op: &str) -> Interp<SnakeVal>
where
    F: Fn(i64, i64) -> (i64, bool),
{
    let n1 = prj_num(v1, "arithmetic", op)?;
    let n2 = prj_num(v2, "arithmetic", op)?;
    let (n3, overflow) = arith(n1, n2);
    if overflow || out_of_bounds(n3) {
        Err(InterpErr::Overflow {
            msg: format!("{} {} {} = {}", n1, op, n2, n3),
        })
    } else {
        Ok(SnakeVal::Num(n3))
    }
}

fn snake_log<F>(v1: SnakeVal, v2: SnakeVal, log: F, op: &str) -> Interp<SnakeVal>
where
    F: Fn(bool, bool) -> bool,
{
    Ok(SnakeVal::Bool(log(
        prj_bool(v1, "logic", op)?,
        prj_bool(v2, "logic", op)?,
    )))
}

fn snake_cmp<F>(v1: SnakeVal, v2: SnakeVal, cmp: F, op: &str) -> Interp<SnakeVal>
where
    F: Fn(i64, i64) -> bool,
{
    Ok(SnakeVal::Bool(cmp(
        prj_num(v1, "comparison", op)?,
        prj_num(v2, "comparison", op)?,
    )))
}

fn interpret_prim2(p: &Prim2, v1: SnakeVal, v2: SnakeVal, heap: &Heap) -> Interp<SnakeVal>
where
{
    match p {
        Prim2::Add => snake_arith(v1, v2, |n1, n2| n1.overflowing_add(n2), "+"),
        Prim2::Sub => snake_arith(v1, v2, |n1, n2| n1.overflowing_sub(n2), "-"),
        Prim2::Mul => snake_arith(v1, v2, |n1, n2| n1.overflowing_mul(n2), "*"),

        Prim2::And => snake_log(v1, v2, |b1, b2| b1 && b2, "&&"),
        Prim2::Or => snake_log(v1, v2, |b1, b2| b1 || b2, "||"),

        Prim2::Lt => snake_cmp(v1, v2, |n1, n2| n1 < n2, "<"),
        Prim2::Le => snake_cmp(v1, v2, |n1, n2| n1 <= n2, "<="),
        Prim2::Gt => snake_cmp(v1, v2, |n1, n2| n1 > n2, ">"),
        Prim2::Ge => snake_cmp(v1, v2, |n1, n2| n1 >= n2, ">="),

        Prim2::Eq => Ok(SnakeVal::Bool(equal_snake_val(&v1, &v2, heap))),
        Prim2::Neq => Ok(SnakeVal::Bool(!equal_snake_val(&v1, &v2, heap))),
        Prim2::ArrayGet => {
            let vs = prj_array(v1, "array index", heap)?;
            let n = valid_index(prj_num(v2, "index", "")?)?;
            match vs.get(n) {
                None => Err(InterpErr::ArrayOutOfBounds {}),
                Some(v) => Ok(*v),
            }
        }
    }
}

/*
 *  Abstract machine-style interpreter.
 *
 *  Defunctionalizes the kontinuation of the direct-style interpreter
 *  so that we don't blow the Rust stack/rely on Rust TCE.
 *
*/
fn machine<'exp, Ann, W>(e: &'exp Exp<Ann>, buf: &mut W, store: &mut State<'exp, Ann>) -> Interp<()>
where
    W: std::io::Write,
    Ann: Clone,
{
    fn call<'exp, Ann>(
        fun_ptr: usize,
        args: Vec<SnakeVal>,
        stk: Stack<'exp, Ann>,
        store: &State<'exp, Ann>,
    ) -> Interp<Machine<'exp, Ann>>
    where
        Ann: Clone,
    {
        let fun = &store.funs[fun_ptr];
        let mut env = fun.closure.env.clone();

        if args.len() != fun.parameters.len() {
            return Err(InterpErr::ArityErr {
                expected_arity: fun.parameters.len(),
                num_provided: args.len(),
            });
        }
        // environment for the body should consist of the captured env
        // extended with the new parameters
        for (v, x) in args.iter().zip(fun.parameters.iter()) {
            env = env.push_local(x.to_string(), *v)
        }
        Ok(Machine::Descending {
            e: fun.closure.exp,
            env,
            stk,
        })
    }

    let mut machine = Machine::Descending {
        e,
        stk: Stack::Done,
        env: Env::new(),
    };
    loop {
        match machine {
            Machine::Descending { e, stk, env } => match e {
                Exp::Num(n, _) => {
                    machine = Machine::Returning {
                        v: SnakeVal::Num(*n),
                        stk,
                    }
                }
                Exp::Bool(b, _) => {
                    machine = Machine::Returning {
                        v: SnakeVal::Bool(*b),
                        stk,
                    }
                }
                Exp::Var(x, _) => {
                    let v = env.lookup(x).expect("Unbound variable in interpreter! You should catch this in the check function!");
                    machine = Machine::Returning { v: *v, stk }
                }
                Exp::Prim1(op, e, _) => {
                    machine = Machine::Descending {
                        e,
                        stk: Stack::Prim1(*op, Box::new(stk)),
                        env,
                    };
                }
                Exp::Prim2(op, e1, e2, _) => {
                    machine = Machine::Descending {
                        e: e1,
                        stk: Stack::Prim2L(
                            *op,
                            Closure {
                                exp: e2,
                                env: env.clone(),
                            },
                            Box::new(stk),
                        ),
                        env,
                    };
                }
                Exp::Let { bindings, body, .. } => {
                    let mut rbindings: Vec<&(String, Exp<Ann>)> = bindings.iter().rev().collect();
                    match rbindings.pop() {
                        None => {
                            machine = Machine::Descending { e: body, stk, env };
                        }
                        Some((var, e)) => {
                            machine = Machine::Descending {
                                e,
                                stk: Stack::Let {
                                    var,
                                    env: env.clone(),
                                    bindings: rbindings,
                                    body,
                                    stk: Box::new(stk),
                                },
                                env,
                            };
                        }
                    }
                }
                Exp::If { cond, thn, els, .. } => {
                    machine = Machine::Descending {
                        e: cond,
                        stk: Stack::If {
                            thn,
                            els,
                            env: env.clone(),
                            stk: Box::new(stk),
                        },
                        env,
                    }
                }

                Exp::Array(es, _) => {
                    let mut r_es: Vec<&Exp<_>> = es.iter().rev().collect();
                    machine = match r_es.pop() {
                        None => {
                            let ptr = store.alloc_array(vec![]);
                            Machine::Returning {
                                v: SnakeVal::Array(ptr),
                                stk,
                            }
                        }
                        Some(e) => Machine::Descending {
                            e,
                            stk: Stack::Array {
                                evaled_parts: Vec::new(),
                                env: env.clone(),
                                remaining_parts: r_es,
                                stk: Box::new(stk),
                            },
                            env,
                        },
                    }
                }
                Exp::ArraySet {
                    array,
                    index,
                    new_value,
                    ..
                } => {
                    machine = Machine::Descending {
                        e: array,
                        env: env.clone(),
                        stk: Stack::ArraySet1 {
                            index,
                            new_value,
                            env,
                            stk: Box::new(stk),
                        },
                    }
                }
                Exp::Semicolon { e1, e2, .. } => {
                    machine = Machine::Descending {
                        e: e1,
                        env: env.clone(),
                        stk: Stack::Semicolon {
                            next: Closure { exp: e2, env },
                            stk: Box::new(stk),
                        },
                    }
                }

                Exp::Call(fun, args, _) => {
                    machine = Machine::Descending {
                        e: fun,
                        stk: Stack::CallFun {
                            args: args.iter().collect(),
                            env: env.clone(),
                            stk: Box::new(stk),
                        },
                        env,
                    }
                }
                Exp::FunDefs { decls, body, .. } => {
                    let env = store.alloc_funs(decls, env);
                    machine = Machine::Descending {
                        e: body,
                        env: env.clone(),
                        stk,
                    }
                }
                Exp::Lambda {
                    parameters, body, ..
                } => {
                    let fun_ptr = store.alloc_fun(parameters, body, env);
                    machine = Machine::Returning {
                        v: SnakeVal::Closure(fun_ptr),
                        stk,
                    }
                }
                Exp::MakeClosure { .. } => {
                    panic!("Shouldn't happen: Interpreter encountered internal form MakeClosure or CallClosure or AssertSize")
                }
            },
            Machine::Returning { v, stk } => match stk {
                Stack::Done => {
                    print_snake_val(buf, v, &store.heap)?;
                    return Ok(());
                }
                Stack::Prim1(op, stk) => {
                    let v = interpret_prim1(&op, buf, v, &store.heap)?;
                    machine = Machine::Returning { v, stk: *stk }
                }
                Stack::Prim2L(op, r, stk) => {
                    machine = Machine::Descending {
                        e: r.exp,
                        env: r.env,
                        stk: Stack::Prim2R(op, v, stk),
                    };
                }
                Stack::Prim2R(op, vl, stk) => {
                    let v = interpret_prim2(&op, vl, v, &store.heap)?;
                    machine = Machine::Returning { v, stk: *stk };
                }
                Stack::Let {
                    var,
                    mut env,
                    mut bindings,
                    body,
                    stk,
                } => {
                    env = env.push_local(var.to_string(), v);
                    machine = match bindings.pop() {
                        None => Machine::Descending {
                            e: body,
                            env,
                            stk: *stk,
                        },
                        Some((var, e)) => Machine::Descending {
                            e,
                            stk: Stack::Let {
                                var,
                                env: env.clone(),
                                bindings,
                                body,
                                stk,
                            },
                            env,
                        },
                    }
                }

                Stack::If { thn, els, env, stk } => {
                    let e = if prj_bool(v, "if", "if")? { thn } else { els };
                    machine = Machine::Descending { e, env, stk: *stk }
                }
                Stack::CallArgs {
                    fun: fun_v,
                    mut evaled_args,
                    env,
                    mut remaining_args,
                    stk,
                } => {
                    evaled_args.push(v);
                    match remaining_args.pop() {
                        None => {
                            machine = call(prj_fun(fun_v)?, evaled_args, *stk, store)?;
                        }
                        Some(e) => {
                            machine = Machine::Descending {
                                e,
                                env: env.clone(),
                                stk: Stack::CallArgs {
                                    fun: fun_v,
                                    evaled_args,
                                    env,
                                    remaining_args,
                                    stk,
                                },
                            }
                        }
                    }
                }
                Stack::CallFun { env, args, stk } => {
                    let mut remaining_args = args;
                    remaining_args.reverse();
                    match remaining_args.pop() {
                        None => {
                            machine = call(prj_fun(v)?, Vec::new(), *stk, store)?;
                        }
                        Some(e) => {
                            machine = Machine::Descending {
                                e,
                                env: env.clone(),
                                stk: Stack::CallArgs {
                                    fun: v,
                                    evaled_args: Vec::new(),
                                    env,
                                    remaining_args,
                                    stk,
                                },
                            }
                        }
                    }
                }
                Stack::Array {
                    mut evaled_parts,
                    mut remaining_parts,
                    env,
                    stk,
                } => {
                    evaled_parts.push(v);
                    machine = match remaining_parts.pop() {
                        None => {
                            let ptr = store.alloc_array(evaled_parts);
                            Machine::Returning {
                                v: SnakeVal::Array(ptr),
                                stk: *stk,
                            }
                        }
                        Some(e) => Machine::Descending {
                            e,
                            env: env.clone(),
                            stk: Stack::Array {
                                evaled_parts,
                                remaining_parts,
                                env,
                                stk,
                            },
                        },
                    }
                }
                Stack::ArraySet1 {
                    index,
                    new_value,
                    env,
                    stk,
                } => {
                    machine = Machine::Descending {
                        e: index,
                        env: env.clone(),
                        stk: Stack::ArraySet2 {
                            array: v,
                            new_value: Closure {
                                exp: new_value,
                                env,
                            },
                            stk,
                        },
                    }
                }
                Stack::ArraySet2 {
                    array,
                    new_value,
                    stk,
                } => {
                    machine = Machine::Descending {
                        e: new_value.exp,
                        env: new_value.env,
                        stk: Stack::ArraySet3 {
                            array,
                            index: v,
                            stk,
                        },
                    }
                }
                Stack::ArraySet3 { array, index, stk } => {
                    let arr = prj_array_mut(array, "array set", &mut store.heap)?;
                    let ix = prj_num(index, "array set", "")?;
                    match TryInto::<usize>::try_into(ix) {
                        Err(_) => return Err(InterpErr::ArrayOutOfBounds {}),
                        Ok(ptr) => match arr.get_mut(ptr) {
                            None => return Err(InterpErr::ArrayOutOfBounds {}),
                            Some(loc) => {
                                *loc = v;
                                machine = Machine::Returning {
                                    v: array,
                                    stk: *stk,
                                }
                            }
                        },
                    }
                }
                Stack::Semicolon { next, stk } => {
                    machine = Machine::Descending {
                        e: next.exp,
                        env: next.env,
                        stk: *stk,
                    }
                }
            },
        }
    }
}

// Runs the reference interpreter.
pub fn exp<Ann, W>(e: &Exp<Ann>, w: &mut W) -> Interp<()>
where
    Ann: Clone,
    W: std::io::Write,
{
    machine(e, w, &mut State::new())
}

pub fn prog<Ann, W>(p: &SurfProg<Ann>, w: &mut W) -> Interp<()>
where
    W: std::io::Write,
    Ann: Clone,
{
    machine(p, w, &mut State::new())
}

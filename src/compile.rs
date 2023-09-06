use crate::asm::instrs_to_string;
use crate::asm::{Arg32, Arg64, BinArgs, Instr, JmpArg, MemRef, MovArgs, Offset, Reg, Reg32};
use crate::lift;
use crate::runtime_error::{
    call_error, check_arity_number, check_array_type, check_bounding, check_closure_type,
    check_field_num, check_if_type, check_index_type, check_method_class, check_overflow,
    check_prim1_type, check_prim2_type,
};
use crate::scope;
use crate::sequence;
use crate::syntax::{
    ClassInfo, Exp, FunDecl, ImmExp, MethodDecl, Prim1, Prim2, SeqExp, SeqProg, SurfProg,
};
use std::collections::HashMap;

static XOR_NOT: u64 = 0x80_00_00_00_00_00_00_00;
static INT_TAG: u64 = 0x00_00_00_00_00_00_00_01;
static TAG_MASK: u64 = 0b111;
static BOOL_TAG: u32 = 0b111;
static ARRAY_TAG: u32 = 0b001;
static CLOSURE_TAG: u32 = 0b011;

static SNAKE_TRUE: u64 = 0xFF_FF_FF_FF_FF_FF_FF_FF;
static SNAKE_FALSE: u64 = 0x7F_FF_FF_FF_FF_FF_FF_FF;

#[derive(Debug, PartialEq, Eq)]
pub enum CompileErr<Span> {
    UnboundVariable {
        unbound: String,
        location: Span,
    },

    UndefinedFunction {
        undefined: String,
        location: Span,
    },

    DuplicateBinding {
        duplicated_name: String,
        location: Span,
    },

    Overflow {
        num: i64,
        location: Span,
    },

    DuplicateFunName {
        duplicated_name: String,
        location: Span,
    },

    DuplicateArgName {
        duplicated_name: String,
        location: Span,
    },

    UndefinedClass {
        undefined: String,
        location: Span,
    },

    UndefinedMethod {
        undefined: String,
        location: Span,
    },

    UndefinedField {
        undefined: String,
        location: Span,
    },

    DuplicateField {
        duplicated_name: String,
        location: Span,
    },

    DuplicateMethod {
        duplicated_name: String,
        location: Span,
    },

    WrongFieldSize {
        class: String,
        location: Span,
    },
}

pub fn check_prog<Span>(p: &SurfProg<Span>) -> Result<(), CompileErr<Span>>
where
    Span: Clone,
{
    scope::check_prog(p, Vec::new(), Vec::new(), Vec::new())
}

fn uniquify(e: &Exp<u32>) -> Exp<()> {
    scope::uniquify(e, Vec::new(), Vec::new(), HashMap::new())
}

// Precondition: all names are uniquified
fn class_lift<Ann>(p: &Exp<Ann>) -> (HashMap<String, ClassInfo>, Exp<()>) {
    lift::class_lift(p, Vec::new(), HashMap::new())
}

// Precondition: all names are uniquified
fn lambda_lift<Ann>(
    p: &Exp<Ann>,
) -> (
    Vec<FunDecl<Exp<()>, ()>>,
    Vec<MethodDecl<Exp<()>, ()>>,
    Exp<()>,
) {
    lift::lambda_lift(p, Vec::new())
}

fn tag_exp<Ann>(p: &SurfProg<Ann>) -> SurfProg<u32> {
    let mut i = 0;
    p.map_ann(
        &mut (|_| {
            let cur = i;
            i += 1;
            cur
        }),
    )
}

fn tag_prog<Ann>(
    defs: &[FunDecl<Exp<Ann>, Ann>],
    methods: &[MethodDecl<Exp<Ann>, Ann>],
    main: &Exp<Ann>,
) -> (
    Vec<FunDecl<Exp<u32>, u32>>,
    Vec<MethodDecl<Exp<u32>, u32>>,
    Exp<u32>,
) {
    let mut i = 0;
    (
        defs.iter()
            .map(|decl| {
                decl.map_ann(
                    &mut (|_| {
                        let cur = i;
                        i += 1;
                        cur
                    }),
                )
            })
            .collect(),
        methods
            .iter()
            .map(|decl| {
                decl.map_ann(
                    &mut (|_| {
                        let cur = i;
                        i += 1;
                        cur
                    }),
                )
            })
            .collect(),
        main.map_ann(
            &mut (|_| {
                let cur = i;
                i += 1;
                cur
            }),
        ),
    )
}

fn tag_sprog<Ann>(p: &SeqProg<Ann>) -> SeqProg<u32> {
    let mut i = 0;
    p.map_ann(
        &mut (|_| {
            let cur = i;
            i += 1;
            cur
        }),
    )
}

// Precondition: expressions do not include local function definitions or lambdas
fn sequentialize_program(
    class_info: HashMap<String, ClassInfo>,
    decls: &[FunDecl<Exp<u32>, u32>],
    methods: &[MethodDecl<Exp<u32>, u32>],
    p: &Exp<u32>,
) -> SeqProg<()> {
    let seq_funs: Vec<FunDecl<SeqExp<()>, ()>> = decls
        .iter()
        .map(|fun| FunDecl {
            name: fun.name.clone(),
            parameters: fun.parameters.clone(),
            body: sequence::sequentialize(&fun.body),
            ann: (),
        })
        .collect();
    let seq_methods: Vec<MethodDecl<SeqExp<()>, ()>> = methods
        .iter()
        .map(|method| MethodDecl {
            class: method.class,
            fundecl: FunDecl {
                name: method.fundecl.name.clone(),
                parameters: method.fundecl.parameters.clone(),
                body: sequence::sequentialize(&method.fundecl.body),
                ann: (),
            },
        })
        .collect();
    SeqProg {
        class: class_info,
        funs: seq_funs,
        methods: seq_methods,
        main: sequence::sequentialize(&p),
        ann: (),
    }
}

fn print_reg(reg: Reg, space: i32) -> Vec<Instr> {
    let mut instr: Vec<Instr> = Vec::new();
    instr.push(Instr::Mov(MovArgs::ToReg(Reg::Rdi, Arg64::Reg(reg))));
    instr.push(Instr::Sub(BinArgs::ToReg(
        Reg::Rsp,
        Arg32::Signed(space + 8),
    )));
    instr.push(Instr::Call(JmpArg::Label(String::from("print_snake_val"))));
    instr.push(Instr::Add(BinArgs::ToReg(
        Reg::Rsp,
        Arg32::Signed(space + 8),
    )));
    instr
}

fn get_offset(env: &HashMap<&str, i32>, x: &str) -> Offset {
    match env.get(x) {
        Some(offset) => Offset::Constant(*offset),
        None => {
            panic!("Variable {} is guaranteed to be in scope", x.clone())
        }
    }
}

fn compile_imm(e: &ImmExp, env: &HashMap<&str, i32>) -> Arg64 {
    match e {
        ImmExp::Num(i) => Arg64::Signed(*i << 1),
        ImmExp::Bool(b) => {
            if *b {
                Arg64::Unsigned(SNAKE_TRUE)
            } else {
                Arg64::Unsigned(SNAKE_FALSE)
            }
        }
        ImmExp::Var(x) => Arg64::Mem(MemRef {
            reg: Reg::Rsp,
            offset: get_offset(env, x),
        }),
    }
}

fn compile_prim1(p: Prim1, space: i32, ann: u32) -> Vec<Instr> {
    let mut instr: Vec<Instr> = Vec::new();
    match p {
        Prim1::Add1 => {
            instr.push(Instr::Comment(String::from("Add1")));
            instr.push(Instr::Add(BinArgs::ToReg(Reg::Rax, Arg32::Signed(1 << 1))));
            instr.extend(check_overflow());
        }
        Prim1::Sub1 => {
            instr.push(Instr::Comment(String::from("Sub1")));
            instr.push(Instr::Sub(BinArgs::ToReg(Reg::Rax, Arg32::Signed(1 << 1))));
            instr.extend(check_overflow());
        }
        Prim1::Not => {
            instr.push(Instr::Comment(String::from("Not")));
            instr.push(Instr::Mov(MovArgs::ToReg(
                Reg::R10,
                Arg64::Unsigned(XOR_NOT),
            )));
            instr.push(Instr::Xor(BinArgs::ToReg(Reg::Rax, Arg32::Reg(Reg::R10))));
        }
        Prim1::Print => {
            instr.push(Instr::Comment(String::from("Print")));
            instr.extend(print_reg(Reg::Rax, space));
        }
        Prim1::IsNum => {
            instr.push(Instr::Comment(String::from("IsNum")));
            instr.push(Instr::Mov(MovArgs::ToReg(
                Reg::R10,
                Arg64::Unsigned(INT_TAG),
            )));
            instr.push(Instr::And(BinArgs::ToReg(Reg::Rax, Arg32::Reg(Reg::R10))));
            instr.push(Instr::Shl(BinArgs::ToReg(Reg::Rax, Arg32::Unsigned(63))));
            instr.push(Instr::Mov(MovArgs::ToReg(
                Reg::R10,
                Arg64::Unsigned(SNAKE_TRUE),
            )));
            instr.push(Instr::Xor(BinArgs::ToReg(Reg::Rax, Arg32::Reg(Reg::R10))));
        }
        Prim1::IsBool | Prim1::IsArray | Prim1::IsFun => {
            instr.push(Instr::Comment(String::from("IsBool/IsArray/IsFun")));
            instr.push(Instr::Mov(MovArgs::ToReg(
                Reg::R10,
                Arg64::Unsigned(TAG_MASK),
            )));
            instr.push(Instr::And(BinArgs::ToReg(Reg::Rax, Arg32::Reg(Reg::R10))));
            instr.push(Instr::Cmp(BinArgs::ToReg(
                Reg::Rax,
                Arg32::Unsigned(match p {
                    Prim1::IsBool => BOOL_TAG,
                    Prim1::IsArray => ARRAY_TAG,
                    Prim1::IsFun => CLOSURE_TAG,
                    _ => panic!("Only IsBool, IsArray, IsFun is supported"),
                }),
            )));
            instr.push(Instr::Mov(MovArgs::ToReg(
                Reg::Rax,
                Arg64::Unsigned(SNAKE_FALSE),
            )));
            instr.push(Instr::Jne(JmpArg::Label(format!("MisMatch_{}", ann))));
            instr.push(Instr::Mov(MovArgs::ToReg(
                Reg::Rax,
                Arg64::Unsigned(SNAKE_TRUE),
            )));
            instr.push(Instr::Label(format!("MisMatch_{}", ann)));
        }
        Prim1::Length => {
            instr.push(Instr::Comment(String::from("Array Length")));
            instr.push(Instr::Sub(BinArgs::ToReg(
                Reg::Rax,
                Arg32::Unsigned(ARRAY_TAG),
            )));
            instr.push(Instr::Mov(MovArgs::ToReg(
                Reg::Rax,
                Arg64::Mem(MemRef {
                    reg: Reg::Rax,
                    offset: Offset::Constant(0),
                }),
            )));
        }
    };
    instr
}

fn compile_prim2(p: Prim2, ann: u32) -> Vec<Instr> {
    let mut instr: Vec<Instr> = Vec::new();
    match p {
        Prim2::Add => {
            instr.push(Instr::Comment(String::from("Add")));
            instr.push(Instr::Add(BinArgs::ToReg(Reg::Rax, Arg32::Reg(Reg::R10))));
            instr.extend(check_overflow());
        }
        Prim2::Sub => {
            instr.push(Instr::Comment(String::from("Sub")));
            instr.push(Instr::Sub(BinArgs::ToReg(Reg::Rax, Arg32::Reg(Reg::R10))));
            instr.extend(check_overflow());
        }
        Prim2::Mul => {
            instr.push(Instr::Comment(String::from("Mul")));
            instr.push(Instr::IMul(BinArgs::ToReg(Reg::Rax, Arg32::Reg(Reg::R10))));
            instr.extend(check_overflow());
            instr.push(Instr::Sar(BinArgs::ToReg(Reg::Rax, Arg32::Unsigned(1))));
        }
        Prim2::And => instr.push(Instr::And(BinArgs::ToReg(Reg::Rax, Arg32::Reg(Reg::R10)))),
        Prim2::Or => instr.push(Instr::Or(BinArgs::ToReg(Reg::Rax, Arg32::Reg(Reg::R10)))),
        Prim2::Lt | Prim2::Gt | Prim2::Le | Prim2::Ge | Prim2::Eq | Prim2::Neq => {
            instr.push(Instr::Comment(String::from("Compare")));
            instr.push(Instr::Cmp(BinArgs::ToReg(Reg::Rax, Arg32::Reg(Reg::R10))));
            instr.push(Instr::Mov(MovArgs::ToReg(
                Reg::Rax,
                Arg64::Unsigned(SNAKE_TRUE),
            )));
            instr.push(match p {
                Prim2::Lt => Instr::Jl(JmpArg::Label(format!("less_than_{}", ann))),
                Prim2::Gt => Instr::Jg(JmpArg::Label(format!("greater_than_{}", ann))),
                Prim2::Le => Instr::Jle(JmpArg::Label(format!("less_equal_{}", ann))),
                Prim2::Ge => Instr::Jge(JmpArg::Label(format!("greater_equal_{}", ann))),
                Prim2::Eq => Instr::Je(JmpArg::Label(format!("equal_{}", ann))),
                Prim2::Neq => Instr::Jne(JmpArg::Label(format!("unequal_{}", ann))),
                _ => panic!("Add, Sub, Mul, And, Or are not supported"),
            });
            instr.push(Instr::Mov(MovArgs::ToReg(
                Reg::Rax,
                Arg64::Unsigned(SNAKE_FALSE),
            )));
            instr.push(Instr::Label(format!(
                "{}_{}",
                match p {
                    Prim2::Lt => "less_than",
                    Prim2::Gt => "greater_than",
                    Prim2::Le => "less_equal",
                    Prim2::Ge => "greater_equal",
                    Prim2::Eq => "equal",
                    Prim2::Neq => "unequal",
                    _ => panic!("Add, Sub, Mul, And, Or are not supported"),
                },
                ann,
            )));
        }
        Prim2::ArrayGet => {
            instr.push(Instr::Comment(String::from("ArrayGet")));
            instr.extend(check_array_type(Reg::Rax));
            instr.push(Instr::Sub(BinArgs::ToReg(
                Reg::Rax,
                Arg32::Unsigned(ARRAY_TAG),
            )));
            instr.extend(check_index_type(Reg::R10));
            instr.extend(check_bounding(Reg::R10, Reg::Rax));
            instr.push(Instr::Mov(MovArgs::ToReg(
                Reg::Rax,
                Arg64::Mem(MemRef {
                    reg: Reg::Rax,
                    offset: Offset::Computed {
                        reg: Reg::R10,
                        factor: 4,
                        constant: 16,
                    },
                }),
            )));
        }
    };
    instr
}

fn compile_to_instrs_help<'exp>(
    e: &'exp SeqExp<u32>,
    mut env: HashMap<&'exp str, i32>,
    class_info: HashMap<String, ClassInfo>,
    space: i32,
    is_tail: bool,
    env_size: usize,
    classidx: usize,
) -> Vec<Instr> {
    let mut instr: Vec<Instr> = Vec::new();
    match e {
        SeqExp::Imm(imm, _) => instr.push(Instr::Mov(MovArgs::ToReg(
            Reg::Rax,
            compile_imm(&imm, &env),
        ))),
        SeqExp::Prim1(p, e, ann) => {
            instr.push(Instr::Comment(String::from("Prim1")));
            instr.push(Instr::Mov(MovArgs::ToReg(Reg::Rax, compile_imm(&e, &env))));
            instr.extend(check_prim1_type(Reg::Rax, &p));
            instr.extend(compile_prim1(*p, space, *ann));
        }
        SeqExp::Prim2(p, e1, e2, ann) => {
            instr.push(Instr::Comment(String::from("Prim2")));
            instr.push(Instr::Mov(MovArgs::ToReg(Reg::Rax, compile_imm(&e1, &env))));
            instr.extend(check_prim2_type(Reg::Rax, &p));
            instr.push(Instr::Mov(MovArgs::ToReg(Reg::R10, compile_imm(&e2, &env))));
            instr.extend(check_prim2_type(Reg::R10, &p));
            instr.extend(compile_prim2(*p, *ann));
        }
        SeqExp::Array(array, _) => {
            instr.push(Instr::Comment(String::from("Array")));
            // Push classidx into heap
            instr.push(Instr::Mov(MovArgs::ToReg(Reg::Rax, Arg64::Unsigned(0))));
            instr.push(Instr::Mov(MovArgs::ToMem(
                MemRef {
                    reg: Reg::R15,
                    offset: Offset::Constant(0),
                },
                Reg32::Reg(Reg::Rax),
            )));
            // Push length of array into heap
            instr.push(Instr::Mov(MovArgs::ToReg(
                Reg::Rax,
                Arg64::Unsigned(2 * array.len() as u64),
            )));
            instr.push(Instr::Mov(MovArgs::ToMem(
                MemRef {
                    reg: Reg::R15,
                    offset: Offset::Constant(8),
                },
                Reg32::Reg(Reg::Rax),
            )));
            // Push array elements into heap
            for (i, element) in array.iter().enumerate() {
                instr.push(Instr::Mov(MovArgs::ToReg(
                    Reg::Rax,
                    compile_imm(&element, &env),
                )));
                instr.push(Instr::Mov(MovArgs::ToMem(
                    MemRef {
                        reg: Reg::R15,
                        offset: Offset::Constant(8 * (i as i32 + 2)),
                    },
                    Reg32::Reg(Reg::Rax),
                )));
            }
            // Store the address of the array in Rax
            instr.push(Instr::Mov(MovArgs::ToReg(Reg::Rax, Arg64::Reg(Reg::R15))));
            instr.push(Instr::Add(BinArgs::ToReg(
                Reg::Rax,
                Arg32::Unsigned(ARRAY_TAG),
            )));
            // Update the heap pointer
            instr.push(Instr::Add(BinArgs::ToReg(
                Reg::R15,
                Arg32::Unsigned(8 * (array.len() as u32 + 2)),
            )));
        }
        SeqExp::ArraySet {
            array,
            index,
            new_value,
            ann: _,
        } => {
            instr.push(Instr::Comment(String::from("ArraySet")));
            // Check classidx of array
            // NYI
            // Set Array Element
            instr.push(Instr::Mov(MovArgs::ToReg(
                Reg::Rax,
                compile_imm(&array, &env),
            )));
            instr.extend(check_array_type(Reg::Rax));
            instr.push(Instr::Sub(BinArgs::ToReg(
                Reg::Rax,
                Arg32::Unsigned(ARRAY_TAG),
            )));
            instr.push(Instr::Mov(MovArgs::ToReg(
                Reg::R10,
                compile_imm(&index, &env),
            )));
            instr.extend(check_index_type(Reg::R10));
            instr.extend(check_bounding(Reg::R10, Reg::Rax));
            instr.push(Instr::Mov(MovArgs::ToReg(
                Reg::Rbx,
                compile_imm(&new_value, &env),
            )));
            instr.push(Instr::Mov(MovArgs::ToMem(
                MemRef {
                    reg: Reg::Rax,
                    offset: Offset::Computed {
                        reg: Reg::R10,
                        factor: 4,
                        constant: 16,
                    },
                },
                Reg32::Reg(Reg::Rbx),
            )));
            instr.push(Instr::Add(BinArgs::ToReg(
                Reg::Rax,
                Arg32::Unsigned(ARRAY_TAG),
            )));
        }
        SeqExp::Let {
            var,
            bound_exp,
            body,
            ann: _,
        } => {
            instr.push(Instr::Comment(String::from("Let")));
            instr.extend(compile_to_instrs_help(
                &bound_exp,
                env.clone(),
                class_info.clone(),
                space,
                false,
                env_size,
                classidx,
            ));
            env.insert(&var, -8 * (env_size as i32 + 1));
            instr.push(Instr::Mov(MovArgs::ToMem(
                MemRef {
                    reg: Reg::Rsp,
                    offset: get_offset(&env, &var),
                },
                Reg32::Reg(Reg::Rax),
            )));
            instr.extend(compile_to_instrs_help(
                &body,
                env.clone(),
                class_info.clone(),
                space,
                is_tail,
                env_size + 1,
                classidx,
            ));
        }
        SeqExp::If {
            cond,
            thn,
            els,
            ann,
        } => {
            instr.push(Instr::Comment(String::from("If")));
            instr.push(Instr::Mov(MovArgs::ToReg(
                Reg::Rax,
                compile_imm(&cond, &env),
            )));
            instr.extend(check_if_type(Reg::Rax));
            instr.push(Instr::Mov(MovArgs::ToReg(
                Reg::R10,
                Arg64::Unsigned(SNAKE_FALSE),
            )));
            instr.push(Instr::Cmp(BinArgs::ToReg(Reg::Rax, Arg32::Reg(Reg::R10))));
            instr.push(Instr::Je(JmpArg::Label(format!("{}_{}", "if_false", ann))));
            instr.extend(compile_to_instrs_help(
                &thn,
                env.clone(),
                class_info.clone(),
                space,
                is_tail,
                env_size,
                classidx,
            ));
            instr.push(Instr::Jmp(JmpArg::Label(format!("{}_{}", "done", ann))));
            instr.push(Instr::Label(format!("{}_{}", "if_false", ann)));
            instr.extend(compile_to_instrs_help(
                &els,
                env.clone(),
                class_info.clone(),
                space,
                is_tail,
                env_size,
                classidx,
            ));
            instr.push(Instr::Label(format!("{}_{}", "done", ann)));
        }
        SeqExp::MakeClosure {
            arity,
            label,
            env: capture,
            ann: _,
        } => {
            instr.push(Instr::Comment(String::from("MakeClosure")));
            // Move arity into heap
            instr.push(Instr::Mov(MovArgs::ToReg(
                Reg::Rax,
                Arg64::Unsigned(*arity as u64),
            )));
            instr.push(Instr::Mov(MovArgs::ToMem(
                MemRef {
                    reg: Reg::R15,
                    offset: Offset::Constant(0),
                },
                Reg32::Reg(Reg::Rax),
            )));
            // Move function pointer into heap
            instr.push(Instr::Mov(MovArgs::ToReg(
                Reg::Rax,
                Arg64::Label(label.clone()),
            )));
            instr.push(Instr::Mov(MovArgs::ToMem(
                MemRef {
                    reg: Reg::R15,
                    offset: Offset::Constant(8),
                },
                Reg32::Reg(Reg::Rax),
            )));
            // Move captured environment into heap
            instr.push(Instr::Mov(MovArgs::ToReg(
                Reg::Rax,
                compile_imm(&capture, &env),
            )));
            instr.push(Instr::Mov(MovArgs::ToMem(
                MemRef {
                    reg: Reg::R15,
                    offset: Offset::Constant(16),
                },
                Reg32::Reg(Reg::Rax),
            )));
            // Return closure address
            instr.push(Instr::Mov(MovArgs::ToReg(Reg::Rax, Arg64::Reg(Reg::R15))));
            instr.push(Instr::Add(BinArgs::ToReg(
                Reg::Rax,
                Arg32::Unsigned(CLOSURE_TAG),
            )));
            // Update the heap pointer
            instr.push(Instr::Add(BinArgs::ToReg(Reg::R15, Arg32::Unsigned(3 * 8))));
        }
        SeqExp::CallClosure { fun, args, ann: _ } => {
            instr.push(Instr::Comment(String::from("CallClosure")));
            // Check closure type
            instr.push(Instr::Mov(MovArgs::ToReg(
                Reg::R10,
                compile_imm(&fun, &env),
            )));
            instr.extend(check_closure_type(Reg::R10));
            instr.push(Instr::Sub(BinArgs::ToReg(
                Reg::R10,
                Arg32::Unsigned(CLOSURE_TAG),
            )));
            // Check arity number
            let arg_num: usize = args.len();
            instr.extend(check_arity_number(Reg::R10, arg_num as u64));
            // Push captured environment as argument to stack
            let mut count: i32 = 16;
            instr.push(Instr::Mov(MovArgs::ToReg(
                Reg::Rax,
                Arg64::Mem(MemRef {
                    reg: Reg::R10,
                    offset: Offset::Constant(16),
                }),
            )));
            instr.push(Instr::Mov(MovArgs::ToMem(
                MemRef {
                    reg: Reg::Rsp,
                    offset: Offset::Constant(-space - count),
                },
                Reg32::Reg(Reg::Rax),
            )));
            count += 8;
            // Push the rest arguments
            for arg in args.iter() {
                instr.push(Instr::Mov(MovArgs::ToReg(
                    Reg::Rax,
                    compile_imm(&arg, &env),
                )));
                instr.push(Instr::Mov(MovArgs::ToMem(
                    MemRef {
                        reg: Reg::Rsp,
                        offset: Offset::Constant(-space - count),
                    },
                    Reg32::Reg(Reg::Rax),
                )));
                count += 8;
            }
            if is_tail {
                instr.push(Instr::Comment(String::from("CallClosure-Tail Recursion")));
                // Move the captured environment
                let mut arg_idx: i32 = 8;
                instr.push(Instr::Mov(MovArgs::ToReg(
                    Reg::Rax,
                    Arg64::Mem(MemRef {
                        reg: Reg::Rsp,
                        offset: Offset::Constant(-space - arg_idx - 8),
                    }),
                )));
                instr.push(Instr::Mov(MovArgs::ToMem(
                    MemRef {
                        reg: Reg::Rsp,
                        offset: Offset::Constant(-arg_idx),
                    },
                    Reg32::Reg(Reg::Rax),
                )));
                arg_idx += 8;
                // Move the arguments
                for _ in args.iter() {
                    instr.push(Instr::Mov(MovArgs::ToReg(
                        Reg::Rax,
                        Arg64::Mem(MemRef {
                            reg: Reg::Rsp,
                            offset: Offset::Constant(-space - arg_idx - 8),
                        }),
                    )));
                    instr.push(Instr::Mov(MovArgs::ToMem(
                        MemRef {
                            reg: Reg::Rsp,
                            offset: Offset::Constant(-arg_idx),
                        },
                        Reg32::Reg(Reg::Rax),
                    )));
                    arg_idx += 8;
                }
                instr.push(Instr::Mov(MovArgs::ToReg(
                    Reg::Rax,
                    Arg64::Mem(MemRef {
                        reg: Reg::R10,
                        offset: Offset::Constant(8),
                    }),
                )));
                instr.push(Instr::Jmp(JmpArg::Reg(Reg::Rax)));
            } else {
                instr.push(Instr::Comment(String::from(
                    "CallClosure-Non Tail Recursion",
                )));
                instr.push(Instr::Sub(BinArgs::ToReg(Reg::Rsp, Arg32::Signed(space))));
                instr.push(Instr::Mov(MovArgs::ToReg(
                    Reg::Rax,
                    Arg64::Mem(MemRef {
                        reg: Reg::R10,
                        offset: Offset::Constant(8),
                    }),
                )));
                instr.push(Instr::Call(JmpArg::Reg(Reg::Rax)));
                instr.push(Instr::Add(BinArgs::ToReg(Reg::Rsp, Arg32::Signed(space))));
            }
        }
        SeqExp::Object {
            class,
            fields,
            ann: _,
        } => {
            instr.push(Instr::Comment(String::from("Object")));
            // Get the class index for the object
            let (classidx, fieldsize) = match class_info.get(&class.clone()) {
                Some(i) => (i.id, i.fieldsize),
                None => panic!("class is guaranteed to be in scope. Compile."),
            };
            instr.extend(check_field_num(fields.len(), fieldsize));
            // Push classidx into heap
            instr.push(Instr::Mov(MovArgs::ToReg(
                Reg::Rax,
                Arg64::Unsigned(classidx as u64),
            )));
            instr.push(Instr::Mov(MovArgs::ToMem(
                MemRef {
                    reg: Reg::R15,
                    offset: Offset::Constant(0),
                },
                Reg32::Reg(Reg::Rax),
            )));
            // Push length of array into heap
            instr.push(Instr::Mov(MovArgs::ToReg(
                Reg::Rax,
                Arg64::Unsigned(2 * fields.len() as u64),
            )));
            instr.push(Instr::Mov(MovArgs::ToMem(
                MemRef {
                    reg: Reg::R15,
                    offset: Offset::Constant(8),
                },
                Reg32::Reg(Reg::Rax),
            )));
            // Push array elements into heap
            for (i, element) in fields.iter().enumerate() {
                instr.push(Instr::Mov(MovArgs::ToReg(
                    Reg::Rax,
                    compile_imm(&element, &env),
                )));
                instr.push(Instr::Mov(MovArgs::ToMem(
                    MemRef {
                        reg: Reg::R15,
                        offset: Offset::Constant(8 * (i as i32 + 2)),
                    },
                    Reg32::Reg(Reg::Rax),
                )));
            }
            // Store the address of the array in Rax
            instr.push(Instr::Mov(MovArgs::ToReg(Reg::Rax, Arg64::Reg(Reg::R15))));
            instr.push(Instr::Add(BinArgs::ToReg(
                Reg::Rax,
                Arg32::Unsigned(ARRAY_TAG),
            )));
            // Update the heap pointer
            instr.push(Instr::Add(BinArgs::ToReg(
                Reg::R15,
                Arg32::Unsigned(8 * (fields.len() as u32 + 2)),
            )));
        }
        SeqExp::CallMethod {
            object,
            method,
            args,
            ann,
        } => {
            instr.push(Instr::Comment(String::from("CallMethod")));
            // Check object type and method type
            instr.push(Instr::Mov(MovArgs::ToReg(
                Reg::Rax,
                compile_imm(&object, &env),
            )));
            instr.extend(check_array_type(Reg::Rax));
            instr.push(Instr::Sub(BinArgs::ToReg(
                Reg::Rax,
                Arg32::Unsigned(ARRAY_TAG),
            )));
            instr.extend(check_method_class(
                Reg::Rax,
                method.clone(),
                class_info,
                *ann,
            ));
            let mut method_name: String = String::from("");
            for (_, method_name_) in method.iter() {
                method_name = method_name_.clone();
                break;
            }
            let fun = ImmExp::Var(method_name);
            // Class index in Rbx
            // Check closure type
            instr.push(Instr::Mov(MovArgs::ToReg(
                Reg::R10,
                compile_imm(&fun, &env),
            )));
            instr.extend(check_closure_type(Reg::R10));
            instr.push(Instr::Sub(BinArgs::ToReg(
                Reg::R10,
                Arg32::Unsigned(CLOSURE_TAG),
            )));
            // Check arity number
            let arg_num: usize = args.len();
            instr.extend(check_arity_number(Reg::R10, (arg_num + 1) as u64));
            // Push captured environment as argument to stack
            let mut count: i32 = 16;
            instr.push(Instr::Mov(MovArgs::ToReg(
                Reg::Rax,
                Arg64::Mem(MemRef {
                    reg: Reg::R10,
                    offset: Offset::Constant(16),
                }),
            )));
            instr.push(Instr::Mov(MovArgs::ToMem(
                MemRef {
                    reg: Reg::Rsp,
                    offset: Offset::Constant(-space - count),
                },
                Reg32::Reg(Reg::Rax),
            )));
            count += 8;
            // Push the object onto the stack
            instr.push(Instr::Mov(MovArgs::ToReg(
                Reg::Rax,
                compile_imm(&object, &env),
            )));
            instr.push(Instr::Mov(MovArgs::ToMem(
                MemRef {
                    reg: Reg::Rsp,
                    offset: Offset::Constant(-space - count),
                },
                Reg32::Reg(Reg::Rax),
            )));
            count += 8;
            // Push the rest arguments
            for arg in args.iter() {
                instr.push(Instr::Mov(MovArgs::ToReg(
                    Reg::Rax,
                    compile_imm(&arg, &env),
                )));
                instr.push(Instr::Mov(MovArgs::ToMem(
                    MemRef {
                        reg: Reg::Rsp,
                        offset: Offset::Constant(-space - count),
                    },
                    Reg32::Reg(Reg::Rax),
                )));
                count += 8;
            }
            if is_tail {
                instr.push(Instr::Comment(String::from("CallClosure-Tail Recursion")));
                // Move the captured environment
                let mut arg_idx: i32 = 8;
                instr.push(Instr::Mov(MovArgs::ToReg(
                    Reg::Rax,
                    Arg64::Mem(MemRef {
                        reg: Reg::Rsp,
                        offset: Offset::Constant(-space - arg_idx - 8),
                    }),
                )));
                instr.push(Instr::Mov(MovArgs::ToMem(
                    MemRef {
                        reg: Reg::Rsp,
                        offset: Offset::Constant(-arg_idx),
                    },
                    Reg32::Reg(Reg::Rax),
                )));
                arg_idx += 8;
                // Move the object
                instr.push(Instr::Mov(MovArgs::ToReg(
                    Reg::Rax,
                    Arg64::Mem(MemRef {
                        reg: Reg::Rsp,
                        offset: Offset::Constant(-space - arg_idx - 8),
                    }),
                )));
                instr.push(Instr::Mov(MovArgs::ToMem(
                    MemRef {
                        reg: Reg::Rsp,
                        offset: Offset::Constant(-arg_idx),
                    },
                    Reg32::Reg(Reg::Rax),
                )));
                arg_idx += 8;
                // Move the arguments
                for _ in args.iter() {
                    instr.push(Instr::Mov(MovArgs::ToReg(
                        Reg::Rax,
                        Arg64::Mem(MemRef {
                            reg: Reg::Rsp,
                            offset: Offset::Constant(-space - arg_idx - 8),
                        }),
                    )));
                    instr.push(Instr::Mov(MovArgs::ToMem(
                        MemRef {
                            reg: Reg::Rsp,
                            offset: Offset::Constant(-arg_idx),
                        },
                        Reg32::Reg(Reg::Rax),
                    )));
                    arg_idx += 8;
                }
                instr.push(Instr::Mov(MovArgs::ToReg(
                    Reg::Rax,
                    Arg64::Mem(MemRef {
                        reg: Reg::R10,
                        offset: Offset::Constant(8),
                    }),
                )));
                instr.push(Instr::Jmp(JmpArg::Reg(Reg::Rax)));
            } else {
                instr.push(Instr::Comment(String::from(
                    "CallClosure-Non Tail Recursion",
                )));
                instr.push(Instr::Sub(BinArgs::ToReg(Reg::Rsp, Arg32::Signed(space))));
                instr.push(Instr::Mov(MovArgs::ToReg(
                    Reg::Rax,
                    Arg64::Mem(MemRef {
                        reg: Reg::R10,
                        offset: Offset::Constant(8),
                    }),
                )));
                instr.push(Instr::Call(JmpArg::Reg(Reg::Rax)));
                instr.push(Instr::Add(BinArgs::ToReg(Reg::Rsp, Arg32::Signed(space))));
            }
        }
    };
    instr
}

fn space_needed_helper<Ann>(e: &SeqExp<Ann>) -> i32 {
    match e {
        SeqExp::Imm(..)
        | SeqExp::Prim1(..)
        | SeqExp::Prim2(..)
        | SeqExp::Array(..)
        | SeqExp::Object { .. }
        | SeqExp::ArraySet { .. }
        | SeqExp::CallMethod { .. }
        | SeqExp::MakeClosure { .. }
        | SeqExp::CallClosure { .. } => 0,
        SeqExp::Let {
            var: _,
            bound_exp,
            body,
            ann: _,
        } => std::cmp::max(
            space_needed_helper(&bound_exp),
            1 + space_needed_helper(&body),
        ),
        SeqExp::If {
            cond: _,
            thn,
            els,
            ann: _,
        } => std::cmp::max(space_needed_helper(&thn), space_needed_helper(&els)),
    }
}

fn space_needed(e: &SeqExp<u32>, arg_num: i32) -> i32 {
    let var_num: i32 = space_needed_helper(&e) + arg_num;
    if var_num % 2 == 0 {
        8 * var_num + 8
    } else {
        8 * var_num
    }
}

fn init_pointers() -> Vec<Instr> {
    let mut instr: Vec<Instr> = Vec::new();
    // Initialize the stack pointer
    // Initialize the heap pointer
    instr.push(Instr::Mov(MovArgs::ToReg(
        Reg::R15,
        Arg64::Label(String::from("HEAP")),
    )));
    instr
}

fn compile_to_instrs(p: &SeqProg<u32>) -> Vec<Instr> {
    let mut instr: Vec<Instr> = Vec::new();
    instr.extend(compile_to_instrs_help(
        &p.main,
        HashMap::new(),
        p.class.clone(),
        space_needed(&p.main, 0),
        true,
        0,
        0,
    ));
    instr.push(Instr::Ret);
    for fun in p.funs.iter() {
        instr.push(Instr::Label(fun.name.clone()));
        let mut env: HashMap<&str, i32> = HashMap::new();
        for (i, arg) in fun.parameters.iter().enumerate() {
            env.insert(&arg, -8 * (i as i32 + 1));
        }
        instr.extend(compile_to_instrs_help(
            &fun.body,
            env,
            p.class.clone(),
            space_needed(&fun.body, fun.parameters.len() as i32),
            true,
            fun.parameters.len(),
            0,
        ));
        instr.push(Instr::Ret);
    }
    for method in p.methods.iter() {
        instr.push(Instr::Label(method.fundecl.name.clone()));
        let mut env: HashMap<&str, i32> = HashMap::new();
        for (i, arg) in method.fundecl.parameters.iter().enumerate() {
            env.insert(&arg, -8 * (i as i32 + 1));
        }
        instr.extend(compile_to_instrs_help(
            &method.fundecl.body,
            env,
            p.class.clone(),
            space_needed(&method.fundecl.body, method.fundecl.parameters.len() as i32),
            true,
            method.fundecl.parameters.len(),
            method.class,
        ));
        instr.push(Instr::Ret);
    }
    instr.extend(call_error());
    instr
}

pub fn compile_to_string<Span>(p: &SurfProg<Span>) -> Result<String, CompileErr<Span>>
where
    Span: Clone,
{
    // first check for errors
    check_prog(p)?;

    // then give all the variables unique names
    let uniq_p = uniquify(&tag_exp(p));

    // lift class information to the top level
    let (class_info, uniq_main) = class_lift(&uniq_p);

    // lift definitions to the top level
    let (defs, methods, main) = lambda_lift(&uniq_main);
    let (t_defs, t_methods, t_main) = tag_prog(&defs, &methods, &main);

    // then sequentialize
    let seq_p = tag_sprog(&sequentialize_program(
        class_info, &t_defs, &t_methods, &t_main,
    ));

    // then codegen
    Ok(format!(
        "\
section .data
HEAP:   times 1024 dq 0
section .text
        global start_here
        extern snake_error
        extern print_snake_val
start_here:
{}        call main
        ret
main:
{}
",
        instrs_to_string(&init_pointers()),
        instrs_to_string(&compile_to_instrs(&seq_p))
    ))
}

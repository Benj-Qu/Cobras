use crate::asm::{Arg32, Arg64, BinArgs, Instr, JmpArg, MemRef, MovArgs, Offset, Reg};
use crate::syntax::{ClassInfo, Prim1, Prim2};

use std::collections::HashMap;

static INT_TAG: u64 = 0x00_00_00_00_00_00_00_01;
static TAG_MASK: u64 = 0b111;
static BOOL_TAG: u32 = 0b111;
static ARRAY_TAG: u32 = 0b001;
static CLOSURE_TAG: u32 = 0b011;

pub enum RuntimeErr {
    IfError,
    CmpError,
    ArithError,
    LogicError,
    OverflowError,
    ArrayError,
    IndexError,
    BoundingError,
    LengthError,
    ClosureError,
    ArityError,
    MethodTypeError,
    FieldNumError,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
    Num,
    Bool,
    Array,
    Closure,
}

pub fn check_overflow() -> Vec<Instr> {
    let mut instr: Vec<Instr> = Vec::new();
    instr.push(Instr::Comment(String::from(
        "Check calculation result overflow",
    )));
    instr.push(Instr::Mov(MovArgs::ToReg(
        Reg::Rdi,
        Arg64::Unsigned(RuntimeErr::OverflowError as u64),
    )));
    instr.push(Instr::Jo(JmpArg::Label(String::from("snake_err"))));
    instr
}

fn check_reg_type_num(reg: Reg, err: RuntimeErr) -> Vec<Instr> {
    let mut instr: Vec<Instr> = Vec::new();
    instr.push(Instr::Comment(String::from("Check Number type")));
    // ready to call snake_error
    instr.push(Instr::Mov(MovArgs::ToReg(
        Reg::Rdi,
        Arg64::Unsigned(err as u64),
    )));
    instr.push(Instr::Mov(MovArgs::ToReg(
        Reg::Rbx,
        Arg64::Unsigned(INT_TAG),
    )));
    instr.push(Instr::Test(BinArgs::ToReg(Reg::Rbx, Arg32::Reg(reg))));
    instr.push(Instr::Jnz(JmpArg::Label(String::from("snake_err"))));
    instr
}

fn check_reg_type_bac(reg: Reg, ty: Type, err: RuntimeErr) -> Vec<Instr> {
    let mut instr: Vec<Instr> = Vec::new();
    instr.push(Instr::Comment(String::from(
        "Check Boolean/Array/Closure type",
    )));
    // ready to call snake_error
    instr.push(Instr::Mov(MovArgs::ToReg(
        Reg::Rdi,
        Arg64::Unsigned(err as u64),
    )));
    instr.push(Instr::Mov(MovArgs::ToReg(
        Reg::Rbx,
        Arg64::Unsigned(TAG_MASK),
    )));
    instr.push(Instr::And(BinArgs::ToReg(Reg::Rbx, Arg32::Reg(reg))));
    instr.push(Instr::Cmp(BinArgs::ToReg(
        Reg::Rbx,
        Arg32::Unsigned(match ty {
            Type::Bool => BOOL_TAG,
            Type::Array => ARRAY_TAG,
            Type::Closure => CLOSURE_TAG,
            Type::Num => panic!("Num type is not supported"),
        }),
    )));
    instr.push(Instr::Jne(JmpArg::Label(String::from("snake_err"))));
    instr
}

fn check_reg_type(reg: Reg, ty: Type, err: RuntimeErr) -> Vec<Instr> {
    match ty {
        Type::Num => check_reg_type_num(reg, err),
        Type::Bool | Type::Array | Type::Closure => check_reg_type_bac(reg, ty, err),
    }
}

pub fn check_prim1_type(reg: Reg, p: &Prim1) -> Vec<Instr> {
    match p {
        Prim1::Add1 | Prim1::Sub1 => check_reg_type(reg, Type::Num, RuntimeErr::ArithError),
        Prim1::Not => check_reg_type(reg, Type::Bool, RuntimeErr::LogicError),
        Prim1::Length => check_reg_type(reg, Type::Array, RuntimeErr::LengthError),
        Prim1::Print | Prim1::IsBool | Prim1::IsNum | Prim1::IsArray | Prim1::IsFun => Vec::new(),
    }
}

pub fn check_prim2_type(reg: Reg, p: &Prim2) -> Vec<Instr> {
    match p {
        Prim2::Lt | Prim2::Gt | Prim2::Le | Prim2::Ge => {
            check_reg_type(reg, Type::Num, RuntimeErr::CmpError)
        }
        Prim2::Add | Prim2::Sub | Prim2::Mul => {
            check_reg_type(reg, Type::Num, RuntimeErr::ArithError)
        }
        Prim2::And | Prim2::Or => check_reg_type(reg, Type::Bool, RuntimeErr::LogicError),
        Prim2::Neq | Prim2::Eq | Prim2::ArrayGet => Vec::new(),
    }
}

pub fn check_if_type(reg: Reg) -> Vec<Instr> {
    check_reg_type(reg, Type::Bool, RuntimeErr::IfError)
}

pub fn check_array_type(reg: Reg) -> Vec<Instr> {
    check_reg_type(reg, Type::Array, RuntimeErr::ArrayError)
}

pub fn check_index_type(reg: Reg) -> Vec<Instr> {
    check_reg_type(reg, Type::Num, RuntimeErr::IndexError)
}

pub fn check_bounding(index_reg: Reg, addr_reg: Reg) -> Vec<Instr> {
    let mut instr: Vec<Instr> = Vec::new();
    instr.push(Instr::Comment(String::from("Check Array Index Bounding")));
    // ready to call snake_error
    instr.push(Instr::Mov(MovArgs::ToReg(
        Reg::Rdi,
        Arg64::Unsigned(RuntimeErr::BoundingError as u64),
    )));
    instr.push(Instr::Cmp(BinArgs::ToReg(
        index_reg,
        Arg32::Mem(MemRef {
            reg: addr_reg,
            offset: Offset::Constant(8),
        }),
    )));
    instr.push(Instr::Jge(JmpArg::Label(String::from("snake_err"))));
    instr.push(Instr::Cmp(BinArgs::ToReg(index_reg, Arg32::Signed(0))));
    instr.push(Instr::Jl(JmpArg::Label(String::from("snake_err"))));
    instr
}

pub fn check_closure_type(reg: Reg) -> Vec<Instr> {
    check_reg_type(reg, Type::Closure, RuntimeErr::ClosureError)
}

pub fn check_arity_number(reg: Reg, arg_num: u64) -> Vec<Instr> {
    let mut instr: Vec<Instr> = Vec::new();
    instr.push(Instr::Comment(String::from("Check Arity Number")));
    instr.push(Instr::Mov(MovArgs::ToReg(
        Reg::Rdi,
        Arg64::Unsigned(RuntimeErr::ArityError as u64),
    )));
    instr.push(Instr::Mov(MovArgs::ToReg(
        Reg::Rbx,
        Arg64::Unsigned(arg_num),
    )));
    instr.push(Instr::Cmp(BinArgs::ToReg(
        Reg::Rbx,
        Arg32::Mem(MemRef {
            reg: reg,
            offset: Offset::Constant(0),
        }),
    )));
    instr.push(Instr::Jne(JmpArg::Label(String::from("snake_err"))));
    instr
}

pub fn check_method_class(
    reg: Reg,
    method_tbl: HashMap<String, String>,
    class_info: HashMap<String, ClassInfo>,
    ann: u32,
) -> Vec<Instr> {
    let mut instr: Vec<Instr> = Vec::new();
    instr.push(Instr::Comment(String::from("Check object and method type")));
    for (class, _) in method_tbl.iter() {
        let classidx = match class_info.get(&class.clone()) {
            Some(i) => i.id,
            None => panic!("Class is guaranteed to be in scope. Error."),
        };
        println!("{}", classidx);
        // ready to call snake_error
        instr.push(Instr::Mov(MovArgs::ToReg(
            Reg::Rdi,
            Arg64::Unsigned(RuntimeErr::MethodTypeError as u64),
        )));
        instr.push(Instr::Mov(MovArgs::ToReg(
            Reg::R11,
            Arg64::Unsigned(classidx as u64),
        )));
        instr.push(Instr::Cmp(BinArgs::ToReg(
            Reg::R11,
            Arg32::Mem(MemRef {
                reg: reg,
                offset: Offset::Constant(0),
            }),
        )));
        instr.push(Instr::Je(JmpArg::Label(format!("Found_{}", ann))));
        break;
    }
    instr.push(Instr::Jmp(JmpArg::Label(String::from("snake_err"))));
    instr.push(Instr::Label(format!("Found_{}", ann)));
    instr
}

pub fn check_field_num(actual_num: usize, correct_num: usize) -> Vec<Instr> {
    let mut instr: Vec<Instr> = Vec::new();
    if actual_num == correct_num {
        return instr;
    }
    // ready to call snake_error
    instr.push(Instr::Mov(MovArgs::ToReg(
        Reg::Rdi,
        Arg64::Unsigned(RuntimeErr::FieldNumError as u64),
    )));
    instr.push(Instr::Jmp(JmpArg::Label(String::from("snake_err"))));
    instr
}

pub fn call_error() -> Vec<Instr> {
    let mut instr: Vec<Instr> = Vec::new();
    instr.push(Instr::Label(String::from("snake_err")));
    instr.push(Instr::Call(JmpArg::Label(String::from("snake_error"))));
    instr
}

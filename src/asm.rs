#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Reg {
    Rax,
    Rbx,
    Rdx,
    Rcx,
    Rsp,
    Rbp,
    Rsi,
    Rdi,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MemRef {
    pub reg: Reg,
    pub offset: Offset,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Offset {
    Constant(i32),
    Computed {
        reg: Reg,
        factor: i32,
        constant: i32,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Arg64 {
    Reg(Reg),
    Signed(i64),
    Unsigned(u64),
    Mem(MemRef),
    Label(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arg32 {
    Reg(Reg),
    Signed(i32),
    Unsigned(u32),
    Mem(MemRef),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reg32 {
    Reg(Reg),
    Signed(i32),
    Unsigned(u32),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MovArgs {
    ToReg(Reg, Arg64),
    ToMem(MemRef, Reg32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinArgs {
    ToReg(Reg, Arg32),
    ToMem(MemRef, Reg32),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JmpArg {
    Label(String),
    Reg(Reg),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Instr {
    Mov(MovArgs),

    Add(BinArgs),
    Sub(BinArgs),
    IMul(BinArgs),
    And(BinArgs),
    Or(BinArgs),
    Xor(BinArgs),
    Shr(BinArgs),
    Sar(BinArgs),
    Shl(BinArgs),
    Cmp(BinArgs),
    Test(BinArgs),

    Push(Arg32),
    Pop(Arg32),

    Comment(String),
    Label(String),

    Call(JmpArg),
    Ret,

    Jmp(JmpArg),
    Je(JmpArg),
    Jne(JmpArg),
    Jl(JmpArg),
    Jle(JmpArg),
    Jg(JmpArg),
    Jge(JmpArg),

    Jz(JmpArg),
    Jnz(JmpArg),
    Jo(JmpArg),
    Jno(JmpArg),
}

pub fn reg_to_string(r: Reg) -> String {
    match r {
        Reg::Rax => String::from("rax"),
        Reg::Rbx => String::from("rbx"),
        Reg::Rcx => String::from("rcx"),
        Reg::Rdx => String::from("rdx"),
        Reg::Rsi => String::from("rsi"),
        Reg::Rdi => String::from("rdi"),
        Reg::Rsp => String::from("rsp"),
        Reg::Rbp => String::from("rbp"),
        Reg::R8 => String::from("r8"),
        Reg::R9 => String::from("r9"),
        Reg::R10 => String::from("r10"),
        Reg::R11 => String::from("r11"),
        Reg::R12 => String::from("r12"),
        Reg::R13 => String::from("r13"),
        Reg::R14 => String::from("r14"),
        Reg::R15 => String::from("r15"),
    }
}

fn imm32_to_string(i: i32) -> String {
    i.to_string()
}

fn offset_to_string(off: Offset) -> String {
    match off {
        Offset::Constant(n) => format!("{}", n),
        Offset::Computed {
            reg,
            factor,
            constant,
        } => format!("{} * {} + {}", reg_to_string(reg), factor, constant),
    }
}
pub fn mem_ref_to_string(m: MemRef) -> String {
    format!(
        "QWORD [{} + {}]",
        reg_to_string(m.reg),
        offset_to_string(m.offset)
    )
}

fn reg32_to_string(r_or_i: Reg32) -> String {
    match r_or_i {
        Reg32::Reg(r) => reg_to_string(r),
        Reg32::Signed(i) => i.to_string(),
        Reg32::Unsigned(u) => format!("0x{:08x}", u),
    }
}

fn arg32_to_string(arg: Arg32) -> String {
    match arg {
        Arg32::Reg(r) => reg_to_string(r),
        Arg32::Signed(i) => imm32_to_string(i),
        Arg32::Unsigned(u) => format!("0x{:08x}", u),
        Arg32::Mem(m) => mem_ref_to_string(m),
    }
}

fn arg64_to_string(arg: &Arg64) -> String {
    match arg {
        Arg64::Reg(r) => reg_to_string(*r),
        Arg64::Signed(i) => i.to_string(),
        Arg64::Unsigned(u) => format!("0x{:016x}", u),
        Arg64::Mem(m) => mem_ref_to_string(*m),
        Arg64::Label(l) => l.clone(),
    }
}

fn mov_args_to_string(args: &MovArgs) -> String {
    match args {
        MovArgs::ToReg(r, arg) => {
            format!("{}, {}", reg_to_string(*r), arg64_to_string(arg))
        }
        MovArgs::ToMem(mem, arg) => {
            format!("{}, {}", mem_ref_to_string(*mem), reg32_to_string(*arg))
        }
    }
}

fn bin_args_to_string(args: BinArgs) -> String {
    match args {
        BinArgs::ToReg(r, arg) => {
            format!("{}, {}", reg_to_string(r), arg32_to_string(arg))
        }
        BinArgs::ToMem(mem, arg) => {
            format!("{}, {}", mem_ref_to_string(mem), reg32_to_string(arg))
        }
    }
}

fn jmp_arg_to_string(arg: &JmpArg) -> String {
    match arg {
        JmpArg::Label(s) => s.clone(),
        JmpArg::Reg(r) => reg_to_string(*r),
    }
}

fn instr_to_string(i: &Instr) -> String {
    match i {
        Instr::Mov(args) => {
            format!("        mov {}", mov_args_to_string(args))
        }
        Instr::Add(args) => {
            format!("        add {}", bin_args_to_string(*args))
        }
        Instr::Sub(args) => {
            format!("        sub {}", bin_args_to_string(*args))
        }
        Instr::Ret => {
            format!("        ret")
        }
        Instr::IMul(args) => {
            format!("        imul {}", bin_args_to_string(*args))
        }
        Instr::And(args) => {
            format!("        and {}", bin_args_to_string(*args))
        }
        Instr::Or(args) => {
            format!("        or {}", bin_args_to_string(*args))
        }
        Instr::Xor(args) => {
            format!("        xor {}", bin_args_to_string(*args))
        }
        Instr::Shr(args) => {
            format!("        shr {}", bin_args_to_string(*args))
        }
        Instr::Sar(args) => {
            format!("        sar {}", bin_args_to_string(*args))
        }
        Instr::Shl(args) => {
            format!("        shl {}", bin_args_to_string(*args))
        }
        Instr::Cmp(args) => {
            format!("        cmp {}", bin_args_to_string(*args))
        }
        Instr::Test(args) => {
            format!("        test {}", bin_args_to_string(*args))
        }
        Instr::Push(arg) => {
            format!("        push {}", arg32_to_string(*arg))
        }
        Instr::Pop(arg) => {
            format!("        pop {}", arg32_to_string(*arg))
        }
        Instr::Label(s) => {
            format!("{}:", s)
        }
        Instr::Comment(s) => {
            format!(";;; {}", s)
        }
        Instr::Jmp(s) => {
            format!("        jmp {}", jmp_arg_to_string(s))
        }
        Instr::Call(s) => {
            format!("        call {}", jmp_arg_to_string(s))
        }
        Instr::Je(s) => {
            format!("        je {}", jmp_arg_to_string(s))
        }
        Instr::Jne(s) => {
            format!("        jne {}", jmp_arg_to_string(s))
        }
        Instr::Jle(s) => {
            format!("        jle {}", jmp_arg_to_string(s))
        }
        Instr::Jl(s) => {
            format!("        jl {}", jmp_arg_to_string(s))
        }
        Instr::Jg(s) => {
            format!("        jg {}", jmp_arg_to_string(s))
        }
        Instr::Jge(s) => {
            format!("        jge {}", jmp_arg_to_string(s))
        }
        Instr::Jz(s) => {
            format!("        jz {}", jmp_arg_to_string(s))
        }
        Instr::Jnz(s) => {
            format!("        jnz {}", jmp_arg_to_string(s))
        }
        Instr::Jo(s) => {
            format!("        jo {}", jmp_arg_to_string(s))
        }
        Instr::Jno(s) => {
            format!("        jno {}", jmp_arg_to_string(s))
        }
    }
}

pub fn instrs_to_string(is: &[Instr]) -> String {
    let mut buf = String::new();
    for i in is {
        buf.push_str(&instr_to_string(&i));
        buf.push_str("\n");
    }
    buf
}

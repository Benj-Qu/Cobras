use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};

use std::fmt::{Display, Formatter};

use crate::compile;
use crate::compile::{compile_to_string, CompileErr};
// use crate::interp;
// use crate::interp::InterpErr;
use crate::parser::ProgParser;
use crate::syntax::SurfProg;

mod span {
    use crate::span::{Span1, Span2};
    use std::fmt::Display;

    #[derive(Clone, Debug)]
    pub struct FileInfo {
        newlines: Vec<usize>,
        len: usize,
    }

    pub fn file_info(s: &str) -> FileInfo {
        FileInfo {
            newlines: s
                .char_indices()
                .filter(|(_i, c)| *c == '\n')
                .map(|(i, _c)| i)
                .collect(),
            len: s.len(),
        }
    }

    pub fn span1_to_span2(info: &FileInfo, offsets: Span1) -> Span2 {
        let mut v = vec![0];
        v.extend(info.newlines.iter().map(|ix| ix + 1));
        v.push(info.len);

        let (start_line, start_col) = offset_to_line_col(&v, offsets.start_ix);
        let (end_line, end_col) = offset_to_line_col(&v, offsets.end_ix - 1);
        Span2 {
            start_line,
            start_col,
            end_line,
            end_col: end_col + 1,
        }
    }

    fn offset_to_line_col(newlines: &[usize], offset: usize) -> (usize, usize) {
        let mut win = newlines.windows(2).enumerate();
        while let Some((line, &[start, end])) = win.next() {
            if start <= offset && offset < end {
                return (line + 1, offset - start);
            }
        }
        panic!("internal error: offset_to_line_col. Send this to the professor");
    }

    use crate::compile::CompileErr;
    impl<Span> CompileErr<Span> {
        pub fn map_span<F, SpanPrime>(self, f: F) -> CompileErr<SpanPrime>
        where
            F: FnOnce(&Span) -> SpanPrime,
        {
            match self {
                CompileErr::UnboundVariable { unbound, location } => CompileErr::UnboundVariable {
                    unbound,
                    location: f(&location),
                },
                CompileErr::DuplicateBinding {
                    duplicated_name,
                    location,
                } => CompileErr::DuplicateBinding {
                    duplicated_name,
                    location: f(&location),
                },
                CompileErr::Overflow { num, location } => CompileErr::Overflow {
                    num,
                    location: f(&location),
                },

                CompileErr::UndefinedFunction {
                    undefined,
                    location,
                } => CompileErr::UndefinedFunction {
                    undefined,
                    location: f(&location),
                },

                CompileErr::DuplicateArgName {
                    duplicated_name,
                    location,
                } => CompileErr::DuplicateArgName {
                    location: f(&location),
                    duplicated_name,
                },

                CompileErr::DuplicateFunName {
                    duplicated_name,
                    location,
                } => CompileErr::DuplicateFunName {
                    duplicated_name,
                    location: f(&location),
                },

                CompileErr::UndefinedClass {
                    undefined,
                    location,
                } => CompileErr::UndefinedClass {
                    undefined: undefined,
                    location: f(&location),
                },

                CompileErr::UndefinedMethod {
                    undefined,
                    location,
                } => CompileErr::UndefinedMethod {
                    undefined: undefined,
                    location: f(&location),
                },

                CompileErr::UndefinedField {
                    undefined,
                    location,
                } => CompileErr::UndefinedField {
                    undefined: undefined,
                    location: f(&location),
                },

                CompileErr::DuplicateField {
                    duplicated_name,
                    location,
                } => CompileErr::DuplicateField {
                    duplicated_name: duplicated_name,
                    location: f(&location),
                },

                CompileErr::DuplicateMethod {
                    duplicated_name,
                    location,
                } => CompileErr::DuplicateMethod {
                    duplicated_name: duplicated_name,
                    location: f(&location),
                },

                CompileErr::WrongFieldSize { class, location } => CompileErr::WrongFieldSize {
                    class: class,
                    location: f(&location),
                },
            }
        }
    }

    impl Display for Span2 {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(
                f,
                "line {}, column {} to line {}, column {}",
                self.start_line, self.start_col, self.end_line, self.end_col
            )
        }
    }
}
use crate::span::{Span1, Span2};
use span::{file_info, span1_to_span2, FileInfo};

pub enum RunnerErr<Span> {
    FileOpen(String),
    Lex(String),
    Parse(String),
    CodeGen(CompileErr<Span>),
    Link(String),
    // Interp(InterpErr),
    Run(String),
}

impl<Span> Display for CompileErr<Span>
where
    Span: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CompileErr::UnboundVariable { unbound, location } => {
                write!(f, "Unbound variable {} at {}", unbound, location)
            }
            CompileErr::UndefinedFunction {
                undefined,
                location,
            } => {
                write!(f, "Undefined function {} called at {}", undefined, location)
            }
            CompileErr::DuplicateBinding {
                duplicated_name,
                location,
            } => write!(
                f,
                "Variable {} defined twice in let-expression at {}",
                duplicated_name, location
            ),

            CompileErr::Overflow { num, location } => write!(
                f,
                "Number literal {} doesn't fit into 63-bit integer at {}",
                num, location
            ),

            CompileErr::DuplicateArgName {
                duplicated_name,
                location,
            } => write!(
                f,
                "multiple arguments named \"{}\" at {}",
                duplicated_name, location,
            ),

            CompileErr::DuplicateFunName {
                duplicated_name,
                location,
            } => write!(
                f,
                "multiple defined functions named \"{}\" at {}",
                duplicated_name, location
            ),

            CompileErr::UndefinedClass {
                undefined,
                location,
            } => write!(
                f,
                "Undefined class \"{}\" applied at {}",
                undefined, location
            ),

            CompileErr::UndefinedMethod {
                undefined,
                location,
            } => write!(
                f,
                "Undefined method \"{}\" called at {}",
                undefined, location
            ),

            CompileErr::UndefinedField {
                undefined,
                location,
            } => write!(
                f,
                "Undefined field \"{}\" modified at {}",
                undefined, location
            ),

            CompileErr::DuplicateField {
                duplicated_name,
                location,
            } => write!(
                f,
                "multiple defined class field named \"{}\" at {}",
                duplicated_name, location
            ),

            CompileErr::DuplicateMethod {
                duplicated_name,
                location,
            } => write!(
                f,
                "multiple defined class methods named \"{}\" at {}",
                duplicated_name, location
            ),

            CompileErr::WrongFieldSize { class, location } => write!(
                f,
                "Wrong number of fields applied to construct object with class \"{}\" at {}",
                class, location
            ),
        }
    }
}

impl<Span> Display for RunnerErr<Span>
where
    Span: Display,
{
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            RunnerErr::FileOpen(s) => write!(f, "Error reading file: {}", s),
            RunnerErr::Lex(s) => write!(f, "Error lexing input: {}", s),
            RunnerErr::Parse(s) => write!(f, "Error parsing input: {}", s),
            RunnerErr::CodeGen(ce) => write!(f, "Error generating assembly: {}", ce),
            RunnerErr::Link(s) => write!(f, "Error linking generated assembly with runtime: {}", s),
            // RunnerErr::Interp(s) => write!(f, "Error in interpreter: {}", s),
            RunnerErr::Run(s) => write!(f, "Error running your compiled output: {}", s),
        }
    }
}

fn fail<Span>(e: RunnerErr<Span>)
where
    Span: Display,
{
    eprintln!("{}", e);
    std::process::exit(1);
}

fn handle_errs<Span>(r: Result<String, RunnerErr<Span>>)
where
    Span: Display,
{
    match r {
        Ok(s) => println!("{}", s),
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}

pub fn emit_assembly(p: &Path) {
    handle_errs(compile_file(p))
}

pub fn run(p: &Path) {
    if let Err(e) = compile_and_run_file(p, Path::new("runtime"), &mut std::io::stdout()) {
        fail(e)
    }
}

pub fn interp<W>(p: &Path, w: &mut W)
where
    W: std::io::Write,
{
    if let Err(e) = interpret_file(p, w) {
        fail(e)
    }
}

pub fn interpret_file<W>(p: &Path, _w: &mut W) -> Result<(), RunnerErr<Span2>>
where
    W: std::io::Write,
{
    let (info, prog) = parse_file(p)?;
    let () = compile::check_prog(&prog)
        .map_err(|e| RunnerErr::CodeGen(e.map_span(|s| span1_to_span2(&info, *s))))?;

    // interp::prog(&prog, w).map_err(|e| RunnerErr::Interp(e))?;
    Ok(())
}

pub fn compile_and_run_file<W>(p: &Path, dir: &Path, out: &mut W) -> Result<(), RunnerErr<Span2>>
where
    W: std::io::Write,
{
    let asm = compile_file(p)?;
    link_and_run(&asm, dir, out)
}

fn compile_file(p: &Path) -> Result<String, RunnerErr<Span2>> {
    let (info, prog) = parse_file(p)?;
    compile_to_string(&prog)
        .map_err(|e| RunnerErr::CodeGen(e.map_span(|s| span1_to_span2(&info, *s))))
}

fn read_file<Span>(p: &Path) -> Result<String, RunnerErr<Span>> {
    let mut f = File::open(p).map_err(|e| RunnerErr::FileOpen(e.to_string()))?;
    let mut buf = String::new();
    f.read_to_string(&mut buf)
        .map_err(|e| RunnerErr::FileOpen(e.to_string()))?;
    Ok(buf)
}

fn parse_file(p: &Path) -> Result<(FileInfo, SurfProg<Span1>), RunnerErr<Span2>> {
    let s = read_file(p)?;
    let e = ProgParser::new()
        .parse(&s)
        .map_err(|e| RunnerErr::Parse(e.to_string()))?;
    Ok((file_info(&s), e))
}

fn link_and_run<W>(assembly: &str, dir: &Path, out: &mut W) -> Result<(), RunnerErr<Span2>>
where
    W: std::io::Write,
{
    let (nasm_format, lib_name) = if cfg!(target_os = "linux") {
        ("elf64", "libcompiled_code.a")
    } else if cfg!(target_os = "macos") {
        ("macho64", "libcompiled_code.a")
    } else if cfg!(target_os = "windows") {
        ("win64", "compiled_code.lib")
    } else {
        panic!("Runner script only works on linux, macos and windows")
    };

    let asm_fname = dir.join("compiled_code.s");
    let obj_fname = dir.join("compiled_code.o");
    let lib_fname = dir.join(lib_name);
    let exe_fname = dir.join("stub.exe");

    // first put the assembly in a new file compiled_code.s
    let mut asm_file = File::create(&asm_fname).map_err(|e| RunnerErr::Link(e.to_string()))?;
    asm_file
        .write(assembly.as_bytes())
        .map_err(|e| RunnerErr::Link(e.to_string()))?;
    asm_file
        .flush()
        .map_err(|e| RunnerErr::Link(e.to_string()))?;

    // nasm -fFORMAT -o compiled_code.o compiled_code.s
    let nasm_out = Command::new("nasm")
        .arg("-f")
        .arg(nasm_format)
        .arg("-o")
        .arg(&obj_fname)
        .arg(&asm_fname)
        .output()
        .map_err(|e| RunnerErr::Link(format!("nasm err: {}", e)))?;
    if !nasm_out.status.success() {
        return Err(RunnerErr::Link(format!(
            "Failure in nasm call: {}\n{}",
            nasm_out.status,
            std::str::from_utf8(&nasm_out.stderr).expect("nasm produced invalid UTF-8")
        )));
    }

    // ar r libcompiled_code.a compiled_code.o
    let ar_out = Command::new("ar")
        .arg("rus")
        .arg(lib_fname)
        .arg(&obj_fname)
        .output()
        .map_err(|e| RunnerErr::Link(format!("ar err: {}", e)))?;
    if !ar_out.status.success() {
        return Err(RunnerErr::Link(format!(
            "Failure in ar call:\n{}\n{}",
            ar_out.status,
            std::str::from_utf8(&ar_out.stderr).expect("ar produced invalid UTF-8")
        )));
    }

    // rustc stub.rs -L tmp
    let rustc_out = if cfg!(target_os = "macos") {
        Command::new("rustc")
            .arg("runtime/stub.rs")
            .arg("--target")
            .arg("x86_64-apple-darwin")
            .arg("-L")
            .arg(dir)
            .arg("-o")
            .arg(&exe_fname)
            .output()
            .map_err(|e| RunnerErr::Link(format!("rustc err: {}", e)))?
    } else {
        Command::new("rustc")
            .arg("runtime/stub.rs")
            .arg("-L")
            .arg(dir)
            .arg("-o")
            .arg(&exe_fname)
            .output()
            .map_err(|e| RunnerErr::Link(format!("rustc err: {}", e)))?
    };
    if !rustc_out.status.success() {
        return Err(RunnerErr::Link(format!(
            "Failure in rustc call: {}\n{}",
            rustc_out.status,
            std::str::from_utf8(&rustc_out.stderr).expect("rustc produced invalid UTF-8")
        )));
    }

    let mut child = Command::new(&exe_fname)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| RunnerErr::Run(format!("{}", e)))?;
    let compiled_out = BufReader::new(
        child
            .stdout
            .take()
            .expect("Failed to capture compiled code's stdout"),
    );
    let compiled_err = BufReader::new(
        child
            .stderr
            .take()
            .expect("Failed to capture compiled code's stderr"),
    );

    for line in compiled_out.lines() {
        let line = line.map_err(|e| RunnerErr::Run(format!("{}", e)))?;
        writeln!(out, "{}", line).map_err(|e| RunnerErr::Run(format!("I/O error: {}", e)))?;
    }

    let status = child
        .wait()
        .map_err(|e| RunnerErr::Run(format!("Error waiting for child process {}", e)))?;
    if !status.success() {
        let mut stderr = String::new();
        for line in compiled_err.lines() {
            stderr.push_str(&format!("{}\n", line.unwrap()));
        }
        return Err(RunnerErr::Run(format!(
            "Error code {} when running compiled code Stderr:\n{}",
            status, stderr
        )));
    }
    Ok(())
}

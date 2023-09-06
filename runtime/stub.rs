use std::collections::HashMap;

#[repr(C)]
#[derive(PartialEq, Eq, Copy, Clone)]
struct SnakeVal(u64);

#[repr(C)]
struct SnakeArray {
    size: u64,
    elts: *const SnakeVal,
}

/* You can use this function to cast a pointer to an array on the heap
 * into something more convenient to access
 *
 */
fn load_snake_array(p: *const u64) -> SnakeArray {
    unsafe {
        let size = *p;
        SnakeArray {
            size,
            elts: std::mem::transmute(p.add(1)),
        }
    }
}

static INT_TAG: u64 = 0x00_00_00_00_00_00_00_01;
static TAG_MASK: u64 = 0b111;
static BOOL_TAG: u64 = 0b111;
static ARRAY_TAG: u64 = 0b001;
static CLOSURE_TAG: u64 = 0b011;

static SNAKE_TRU: SnakeVal = SnakeVal(0xFF_FF_FF_FF_FF_FF_FF_FF);
static SNAKE_FLS: SnakeVal = SnakeVal(0x7F_FF_FF_FF_FF_FF_FF_FF);

enum RuntimeErr {
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

#[link(name = "compiled_code", kind = "static")]
extern "sysv64" {

    // The \x01 here is an undocumented feature of LLVM that ensures
    // it does not add an underscore in front of the name.
    #[link_name = "\x01start_here"]
    fn start_here() -> SnakeVal;
}

// reinterprets the bytes of an unsigned number to a signed number
fn unsigned_to_signed(x: u64) -> i64 {
    i64::from_le_bytes(x.to_le_bytes())
}

fn sprint_snake_val_helper(x: SnakeVal, mut arr_env: HashMap<u64, ()>) -> String {
    if x.0 & INT_TAG == 0 {
        // it's a number
        format!("{}", unsigned_to_signed(x.0) >> 1)
    } else if x == SNAKE_TRU {
        String::from("true")
    } else if x == SNAKE_FLS {
        String::from("false")
    } else if x.0 & TAG_MASK == ARRAY_TAG {
        // it's an array
        match arr_env.get(&x.0) {
            Some(()) => return String::from("<loop>"),
            None => arr_env.insert(x.0, ()),
        };
        let address = x.0 - ARRAY_TAG + 8;
        let pointer = address as *const u64;
        let arr_len = unsafe { *pointer >> 1 };
        let mut arr: String = String::from("[");
        for i in 1..=arr_len {
            let element = unsafe { *((address + i * 8) as *const u64) };
            if i == 1 {
                arr = format!(
                    "{}{}",
                    arr,
                    sprint_snake_val_helper(SnakeVal(element), arr_env.clone())
                );
            } else {
                arr = format!(
                    "{}, {}",
                    arr,
                    sprint_snake_val_helper(SnakeVal(element), arr_env.clone())
                );
            }
        }
        arr = format!("{}{}", arr, "]");
        format!("{}", arr)
    } else if x.0 & TAG_MASK == CLOSURE_TAG {
        // it's a closure
        String::from("<closure>")
    } else {
        format!("Invalid snake value 0x{:x}", x.0)
    }
}

fn sprint_snake_val(x: SnakeVal) -> String {
    sprint_snake_val_helper(x, HashMap::new())
}

#[export_name = "\x01print_snake_val"]
extern "sysv64" fn print_snake_val(v: SnakeVal) -> SnakeVal {
    println!("{}", sprint_snake_val(v));
    return v;
}

/* Implement the following error function. You are free to change the
 * input and output types as needed for your design.
 *
**/
#[export_name = "\x01snake_error"]
extern "sysv64" fn snake_error(err_code: u64) {
    if err_code == (RuntimeErr::IfError as u64) {
        eprintln!("if expected a boolean");
    } else if err_code == (RuntimeErr::CmpError as u64) {
        eprintln!("comparison expected a number");
    } else if err_code == (RuntimeErr::ArithError as u64) {
        eprintln!("arithmetic expected a number");
    } else if err_code == (RuntimeErr::LogicError as u64) {
        eprintln!("logic expected a boolean");
    } else if err_code == (RuntimeErr::OverflowError as u64) {
        eprintln!("overflow");
    } else if err_code == (RuntimeErr::ArrayError as u64) {
        eprintln!("indexed into non-array");
    } else if err_code == (RuntimeErr::IndexError as u64) {
        eprintln!("index not a number");
    } else if err_code == (RuntimeErr::BoundingError as u64) {
        eprintln!("index out of bounds");
    } else if err_code == (RuntimeErr::LengthError as u64) {
        eprintln!("length called with non-array");
    } else if err_code == (RuntimeErr::ClosureError as u64) {
        eprintln!("called a non-function");
    } else if err_code == (RuntimeErr::ArityError as u64) {
        eprintln!("wrong number of arguments");
    } else if err_code == (RuntimeErr::MethodTypeError as u64) {
        eprintln!("calling method from another class");
    } else if err_code == (RuntimeErr::FieldNumError as u64) {
        eprintln!("wrong number of fields when constructing object");
    } else {
        eprintln!("Unknown Error!");
    }
    std::process::exit(1);
}

fn main() {
    let output = unsafe { start_here() };
    println!("{}", sprint_snake_val(output));
}

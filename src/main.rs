use snake::runner::*;
use std::path::Path;

static USAGE_MSG: &str = "\
To compile a program and emit assembly code to stdout use

    snake INPUT_FILE

To compile a program, link it and run the produced binary use

    snake --run INPUT_FILE

To run the reference interpreter use

    snake --interp INPUT_FILE

To see this usage message run

    snake --help
";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 2 {
        match args[1].as_str() {
            "--interp" | "--run" => usage(Some("interp and run comands require an input file")),
            "--help" => usage(None),
            path => emit_assembly(Path::new(&path)),
        }
    } else if args.len() == 3 {
        match args[1].as_str() {
            "--interp" => interp(Path::new(&args[2]), &mut std::io::stdout()),
            "--run" => run(Path::new(&args[2])),
            _ => usage(Some("Failed to parse input")),
        }
    } else if args.len() <= 1 {
        usage(Some("Too few arguments"))
    } else {
        usage(Some("Too many arguments"))
    }
}

fn usage(err_msg: Option<&str>) {
    match err_msg {
        None => println!("{}", USAGE_MSG),
        Some(e) => {
            eprintln!("{}", e);
            eprintln!("{}", USAGE_MSG);
            std::process::exit(1);
        }
    }
}

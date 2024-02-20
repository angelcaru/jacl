mod codegen;
mod ir;
mod lexer;
mod loc;
mod parser;

//use shell_quote::{Bash, QuoteRefExt};
use std::{
    env,
    fs::{read_dir, DirEntry, File},
    io::{Error, Read},
    os::unix::process::ExitStatusExt,
    process::{exit, Command, ExitStatus},
};

use codegen::x86_64::Compile;
use ir::Program;
use lexer::Lexer;
use parser::parse;

use crate::parser::ParseError;

fn read_file(name: &String) -> std::io::Result<String> {
    let mut txt = String::new();
    let mut file = File::open(name)?;
    file.read_to_string(&mut txt)?;
    Ok(txt)
}

fn run_cmd(cmd: &[String]) -> std::io::Result<ExitStatus> {
    print!("[CMD] ");
    for arg in cmd {
        let arg = arg.clone();
        //let quoted: String = arg.quoted(Bash);
        let quoted = if arg.contains(' ') {
            format!("\"{arg}\"")
        } else {
            arg
        };
        print!("{} ", quoted);
    }
    print!("\n");

    let mut command = Command::new(cmd[0].clone());
    for arg in &cmd[1..] {
        command.arg(arg);
    }
    let mut process = command.spawn()?;
    process.wait()
}

fn parse_debug_flag(args: &mut std::iter::Peekable<env::Args>) -> bool {
    match args.peek() {
        Some(s) if s == "--debug" => {
            args.next();
            true
        }
        _ => false,
    }
}

fn parse_and_report_err(lexer: Lexer<std::str::Chars<'_>>, debug: bool) -> parser::Node {
    parse(lexer, debug)
        .inspect_err(|err| match err {
            ParseError::Error(loc, err) => {
                eprintln!("{}: {}", loc, err);
                exit(1);
            }
            ParseError::LexerError(loc, err) => {
                eprintln!("{}: {}", loc, err);
                exit(1);
            }
            _ => panic!("unreachable"),
        })
        .unwrap()
}

fn generate_ir_and_report_err(ast: parser::Node) -> Program {
    Program::from_ast(&ast)
        .inspect_err(|err| {
            eprintln!("{}", err);
            exit(1);
        })
        .unwrap()
}

fn add_prefix(prefix: &'static str) -> impl FnMut(String) -> String {
    move |x| format!("{prefix}{x}")
}

fn remove_bs(bs: Result<DirEntry, Error>) -> String {
    bs.unwrap().file_name().into_string().unwrap()
}

fn compile_prog(prog: Program, binary_path: &String) -> Result<(), std::io::Error> {
    //set_current_dir("./asm")?;
    prog.compile_to_asm("out.asm")?;
    for file_name in read_dir("std/")?.map(remove_bs).map(add_prefix("std/")) {
        if file_name.ends_with(".asm") {
            let code = run_cmd(&["fasm".into(), file_name.into()])?;
            if !code.success() {
                eprintln!("[ERROR] fasm exited with code {}", code.into_raw());
                exit(1);
            }
        }
    }
    let code = run_cmd(&["fasm".into(), "out.asm".into()])?;
    if !code.success() {
        eprintln!("[ERROR] fasm exited with code {}", code.into_raw());
        exit(1);
    }

    let mut args = vec!["ld".into(), "out.o".into()];
    args.extend(
        read_dir("std/")?
            .map(remove_bs)
            .map(add_prefix("std/"))
            .filter(|x| x.ends_with(".o"))
            .collect::<Vec<_>>(),
    );
    args.extend(["-o".into(), binary_path.clone()]);
    let code = run_cmd(&args[..])?;
    Ok(if !code.success() {
        eprintln!("[ERROR] ld exited with code {}", code.into_raw());
        exit(1);
    })
}

fn main() -> std::io::Result<()> {
    let mut args = env::args().peekable();
    let _program = args.next().expect("Program name");
    let debug = parse_debug_flag(&mut args);
    let filename = args.next().expect("Please provide a program");
    let code = read_file(&filename)?;

    let lexer = Lexer::from_iter(&filename, code.chars());

    let ast = parse_and_report_err(lexer, debug);
    if debug {
        println!("{ast:#?}");
    }

    let prog = generate_ir_and_report_err(ast);

    if debug {
        prog.disassemble();
    }

    let binary_path = &"./test".into();
    compile_prog(prog, binary_path)?;

    println!("[INFO] Success! Finished binary is at {binary_path}");

    Ok(())
}

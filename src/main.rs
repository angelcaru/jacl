mod codegen;
mod ir;
mod lexer;
mod loc;
mod parser;

use shell_quote::{Bash, QuoteRefExt};
use std::{
    env::{self, set_current_dir},
    fs::{read_dir, File},
    io::Read,
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

#[allow(suspicious_double_ref_op)] // That's what I wanted to do, idiot compiler!
fn run_cmd(cmd: &[String]) -> std::io::Result<ExitStatus> {
    print!("[CMD] ");
    for arg in cmd {
        let arg = arg.clone();
        let quoted: String = arg.quoted(Bash);
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

fn main() -> std::io::Result<()> {
    let mut args = env::args();
    let _program = args.next().expect("Program name");

    let filename = args.next().expect("Please provide a program");
    let code = read_file(&filename)?;

    let lexer = Lexer::from_iter(&filename, code.chars());
    let ast = parse(lexer).inspect_err(|err| {
        if let ParseError::Error(loc, err) = err {
            eprintln!("{}: {}", loc, err);
            exit(1);
        } else {
            panic!("unreachable");
        }
    }).unwrap();
    println!("{ast:?}");
    let prog = Program::from_ast(&ast);

    prog.disassemble();

    set_current_dir("./asm")?;
    prog.compile_to_asm("out.asm")?;

    for file in read_dir(".")? {
        let file_name = file?.file_name();
        let file_name = file_name.to_str().unwrap();
        if file_name.ends_with(".asm") {
            let code = run_cmd(&["fasm".into(), file_name.into()])?;
            if !code.success() {
                eprintln!("[ERROR] fasm exited with code {}", code.into_raw());
                exit(1);
            }
        }
    }

    let mut args = vec!["ld".into()];
    for file in read_dir(".")? {
        let file_name = file?.file_name();
        let file_name = file_name.to_str().unwrap();
        if file_name.ends_with(".o") {
            args.push(file_name.to_string());
        }
    }
    
    args.extend(["-o".into(), "test".into()]);
    let code = run_cmd(&args[..])?;
    if !code.success() {
        eprintln!("[ERROR] ld exited with code {}", code.into_raw());
        exit(1);
    }
    
    println!("[INFO] Success! Finished binary is at asm/test");

    Ok(())
}

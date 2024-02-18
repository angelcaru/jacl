mod ir;
mod lexer;
mod loc;
mod parser;
mod codegen;

use std::{env, fs::File, io::Read};

use codegen::x86_64::Compile;
use ir::Program;
use lexer::Lexer;
use parser::parse;

fn read_file(name: &String) -> std::io::Result<String> {
    let mut txt = String::new();
    let mut file = File::open(name)?;
    file.read_to_string(&mut txt)?;
    Ok(txt)
}

fn main() -> std::io::Result<()> {
    let mut args = env::args();
    let _program = args.next().expect("Program name");

    let filename = args.next().expect("Please provide a program");
    let code = read_file(&filename)?;

    let lexer = Lexer::from_iter(&filename, code.chars());
    let ast = parse(lexer).unwrap();
    println!("{ast:?}");
    let prog = Program::from_ast(&ast);

    prog.disassemble();
    prog.compile_to_asm("asm/out.asm")?;

    Ok(())
}

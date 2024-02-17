mod lexer;
mod loc;
mod parser;

use std::{env, fs::File, io::Read};

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

    match parse(lexer) {
        Ok(node) => println!("{node:?}"),
        Err(err) => eprintln!("{err:?}"),
    }

    Ok(())
}

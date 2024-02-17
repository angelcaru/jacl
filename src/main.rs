mod lexer;
mod parser;

use std::{
    env,
    fmt::{Debug, Display, Formatter},
    fs::File,
    io::Read,
};

use lexer::Lexer;
use parser::parse;

fn read_file(name: &String) -> std::io::Result<String> {
    let mut txt = String::new();
    let mut file = File::open(name)?;
    file.read_to_string(&mut txt)?;
    Ok(txt)
}

#[derive(Clone, Copy, PartialEq)]
struct Loc<'a> {
    path: &'a String,
    line: usize,
    col: usize,
    char: usize,
}

impl Debug for Loc<'_> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(fmt, "{self}")
    }
}

impl Display for Loc<'_> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(fmt, "{}:{}:{}", self.path, self.line, self.col)
    }
}

impl<'a> Loc<'a> {
    fn new(path: &'a String) -> Self {
        Self {
            path,
            line: 1,
            col: 1,
            char: 0,
        }
    }

    // FIXME: line reporting is off on the first character of a line
    fn advance(&mut self, ch: char) -> char {
        self.char += 1;
        if ch == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        ch
    }
}

#[allow(dead_code)]
fn logpoint<T: Display>(val: T) -> T {
    println!("{val}");
    val
}

fn main() -> std::io::Result<()> {
    let mut args = env::args();
    let _program = args.next().expect("Program name");

    let filename = args.next().expect("Please provide a program");
    let code = read_file(&filename)?;

    let lexer = Lexer::from_iter(&filename, code.chars());

    match parse(lexer) {
        Ok(node) => println!("{node:?}"),
        Err(err) => eprintln!("{err:?}")
    }

    Ok(())
}

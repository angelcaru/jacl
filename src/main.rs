use std::{env, fs::File, io::Read};

fn read_file(name: String) -> std::io::Result<String> {
    let mut txt = String::new();
    let mut file = File::open(name)?;
    file.read_to_string(&mut txt)?;
    Ok(txt)
}

#[derive(Debug)]
enum Token {
    Name(String),
    LParen,
    RParen,
    StrLit(String),
    Semicolon,
}

struct Lexer<T: DoubleEndedIterator<Item = char>> {
    code: T,
}

impl<T: Iterator<Item = char>> Lexer<T> {
    fn from_iter(iter: T) -> Self {
        Self { code: iter }
    }
}

impl<T: Iterator<Item = char>> Iterator for Lexer<T> {
    type Item = Token;
    fn next(&mut self) -> Option<Token> {
        Some(match self.code.next()? {
            '(' => Token::LParen,
            ')' => Token::RParen,
            ';' => Token::Semicolon,
            ch if ch.is_alphabetic() => {
                let mut name = String::new();

                name.push(ch);
                while let Some(ch) = self.code.next() {
                    if !ch.is_alphanumeric() {
                        break;
                    }
                    name.push(ch);
                }

                Token::Name(name)
            }
            '"' => {
                todo!()
            }
            ch if ch.is_whitespace() => self.next()?, // ignore

            ch => panic!("Invalid char: {ch}"),
        })
    }
}

fn main() -> std::io::Result<()> {
    let mut args = env::args();
    let _program = args.next().expect("Program name");

    let filename = args.next().expect("Please provide a program");
    let code = read_file(filename)?;

    for tok in Lexer::from_iter(code.chars()) {
        println!("{tok:?}");
    }

    Ok(())
}

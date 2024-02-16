use std::{
    env,
    fmt::{Debug, Display, Formatter},
    fs::File,
    io::Read,
    iter::Peekable,
};

fn read_file(name: &String) -> std::io::Result<String> {
    let mut txt = String::new();
    let mut file = File::open(name)?;
    file.read_to_string(&mut txt)?;
    Ok(txt)
}

#[derive(Clone, Copy)]
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

#[derive(Debug)]
struct Token<'a> {
    loc: Loc<'a>,
    data: TokenData,
}

#[derive(Debug)]
enum TokenData {
    Name(String),
    LParen,
    RParen,
    StrLit(String),
    Semicolon,
}

struct Lexer<'a, T: Iterator<Item = char>> {
    loc: Loc<'a>,
    code: Peekable<T>,
}

impl<'a, T: Iterator<Item = char>> Lexer<'a, T> {
    fn from_iter(path: &'a String, iter: T) -> Self {
        Self {
            loc: Loc::new(path),
            code: iter.peekable(),
        }
    }
}

#[allow(dead_code)]
fn logpoint<T: Display>(val: T) -> T {
    println!("{val}");
    val
}

impl<'a, T: Iterator<Item = char>> Iterator for Lexer<'a, T> {
    type Item = Token<'a>;
    fn next(&mut self) -> Option<Token<'a>> {
        Some(Token {
            loc: self.loc,
            data: match self.loc.advance(self.code.next()?) {
                '(' => TokenData::LParen,
                ')' => TokenData::RParen,
                ';' => TokenData::Semicolon,
                ch if ch.is_alphabetic() => {
                    let mut name = String::new();

                    name.push(ch);
                    while let Some(ch) = self.code.peek() {
                        if !ch.is_alphanumeric() {
                            break;
                        }
                        name.push(self.loc.advance(self.code.next().expect("We were able to peek tho")));
                    }

                    TokenData::Name(name)
                }
                '"' => {
                    let mut string = String::new();

                    while let Some(ch) = self.code.next() {
                        self.loc.advance(ch);
                        if ch == '"' {
                            break;
                        }
                        string.push(ch);
                    }

                    TokenData::StrLit(string)
                }
                ch if ch.is_whitespace() => self.next()?.data, // ignore

                ch => panic!("Invalid char: {ch}"),
            },
        })
    }
}

fn main() -> std::io::Result<()> {
    let mut args = env::args();
    let _program = args.next().expect("Program name");

    let filename = args.next().expect("Please provide a program");
    let code = read_file(&filename)?;

    for tok in Lexer::from_iter(&filename, code.chars()) {
        println!("{tok:?}");
    }

    Ok(())
}

use std::iter::Peekable;

use crate::loc::Loc;

#[derive(Debug, PartialEq)]
pub struct Token {
    pub loc: Loc,
    pub data: TokenData,
}

#[derive(Debug, PartialEq)]
pub enum TokenData {
    Name(String),
    LParen,
    RParen,
    StrLit(String),
    Semicolon,
    Let,
    Equals,
    Int(usize),
    Plus,
    Minus,
    Mult,
    Div,
    Less,
    EqEq,
    Greater,
    LtEq,
    GtEq,
    If,
    LCurly,
    RCurly,
    Else,
    Unless,
    While,
    Comma,
}

pub struct Lexer<T: Iterator<Item = char>> {
    loc: Loc,
    code: Peekable<T>,
}

impl<'a, T: Iterator<Item = char>> Lexer<T> {
    pub fn from_iter(path: &String, iter: T) -> Self {
        Self {
            loc: Loc::new(path),
            code: iter.peekable(),
        }
    }
}

fn keyword_or_name(name: &str) -> TokenData {
    match name {
        "let" => TokenData::Let,
        "if" => TokenData::If,
        "else" => TokenData::Else,
        "unless" => TokenData::Unless,
        "while" => TokenData::While,
        name => TokenData::Name(name.into()),
    }
}

impl<T: Iterator<Item = char>> Iterator for Lexer<T> {
    type Item = Result<Token, (Loc, String)>;
    fn next(&mut self) -> Option<Result<Token, (Loc, String)>> {
        Some(Ok(Token {
            loc: self.loc.clone(),
            data: match self.loc.advance(self.code.next()?) {
                '(' => TokenData::LParen,
                ')' => TokenData::RParen,
                ';' => TokenData::Semicolon,
                '+' => TokenData::Plus,
                '-' => TokenData::Minus,
                '*' => TokenData::Mult,
                '{' => TokenData::LCurly,
                '}' => TokenData::RCurly,
                ',' => TokenData::Comma,
                '=' => {
                    if let Some('=') = self.code.peek() {
                        self.loc.advance(self.code.next().unwrap());
                        TokenData::EqEq
                    } else {
                        TokenData::Equals
                    }
                }
                '<' => {
                    if let Some('=') = self.code.peek() {
                        self.loc.advance(self.code.next().unwrap());
                        TokenData::LtEq
                    } else {
                        TokenData::Less
                    }
                }
                '>' => {
                    if let Some('=') = self.code.peek() {
                        self.loc.advance(self.code.next().unwrap());
                        TokenData::GtEq
                    } else {
                        TokenData::Greater
                    }
                }
                '/' => {
                    if let Some('/') = self.code.peek() {
                        while self.loc.advance(self.code.next()?) != '\n' {}
                        return self.next();
                    } else {
                        TokenData::Div
                    }
                }
                ch if ch.is_alphabetic() || ch == '_' => {
                    let mut name = String::new();

                    name.push(ch);
                    while let Some(&ch) = self.code.peek() {
                        if !(ch.is_alphanumeric() || ch == '_') {
                            break;
                        }
                        name.push(
                            self.loc
                                .advance(self.code.next().expect("We were able to peek tho")),
                        );
                    }

                    keyword_or_name(&name)
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
                ch if ch.is_digit(10) => {
                    let mut number = String::new();

                    number.push(ch);
                    while let Some(ch) = self.code.peek() {
                        if !ch.is_digit(10) {
                            break;
                        }
                        number.push(
                            self.loc
                                .advance(self.code.next().expect("We were able to peek tho")),
                        );
                    }

                    TokenData::Int(number.parse().unwrap())
                }
                ch if ch.is_whitespace() => return self.next(), // ignore

                ch => {
                    return Some(Err((
                        self.loc.clone(),
                        format!("Invalid char: {ch}").into(),
                    )))
                }
            },
        }))
    }
}

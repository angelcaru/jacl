use std::iter::Peekable;

use crate::loc::Loc;

#[derive(Debug, PartialEq)]
pub struct Token<'a> {
    pub loc: Loc<'a>,
    pub data: TokenData,
}

#[derive(Debug, PartialEq)]
pub enum TokenData {
    Name(String),
    LParen,
    RParen,
    StrLit(String),
    Semicolon,
}

pub struct Lexer<'a, T: Iterator<Item = char>> {
    loc: Loc<'a>,
    code: Peekable<T>,
}

impl<'a, T: Iterator<Item = char>> Lexer<'a, T> {
    pub fn from_iter(path: &'a String, iter: T) -> Self {
        Self {
            loc: Loc::new(path),
            code: iter.peekable(),
        }
    }
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
                        name.push(
                            self.loc
                                .advance(self.code.next().expect("We were able to peek tho")),
                        );
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

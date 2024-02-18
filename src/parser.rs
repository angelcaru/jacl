use crate::lexer::{Token, TokenData};

pub type NodeList = Vec<Node>;

#[derive(Debug, Clone, Copy)]
pub enum BinOp {
    Plus,
    Minus,
    Mult,
    Div
}

#[derive(Debug)]
pub enum Node {
    FuncCall(String, NodeList),
    StrLit(String),
    Block(NodeList),
    VarDecl(String, Box<Node>),
    VarAccess(String),
    VarAssign(String, Box<Node>),
    Int(usize),
    BinOp(BinOp, Box<Node>, Box<Node>),
}

// TODO: Provide details for parse error
#[derive(Debug)]
pub struct ParseError(String);

type ParseResult<T> = Result<T, ParseError>;

pub fn parse<'a, T: Iterator<Item = Token<'a>>>(lexer: T) -> ParseResult<Node> {
    Parser {
        lexer: lexer.collect(),
        i: 0,
    }
    .parse_block()
}

struct Parser<'a> {
    lexer: Vec<Token<'a>>,
    i: usize,
}

/*
macro_rules! expect_pattern {
    ($($pattern:tt)*) => {
        self.expect_fn(|tok| match tok {
            $pattern => true,
            _ => false
        })
    };
}
*/

impl<'a> Parser<'a> {
    fn parse_block(&mut self) -> ParseResult<Node> {
        println!("{:#?}", self.lexer);
        let mut statements = Vec::new();

        let st = self.parse_statement()?;
        statements.push(st);

        while let Some(TokenData::Semicolon) = self.nom() {
            if self.is_empty() {
                break;
            }
            let st = self.parse_statement()?;
            statements.push(st);
        }

        Ok(Node::Block(statements))
    }

    fn parse_statement(&mut self) -> ParseResult<Node> {
        match self.nom().ok_or(ParseError(line!().to_string()))? {
            TokenData::Name(name) => {
                let name = name.clone();

                match self.nom() {
                    Some(TokenData::LParen) => {
                        let args = vec![self.parse_expr()?];
                        self.expect(TokenData::RParen)?;
                        Ok(Node::FuncCall(name, args))
                    }
                    Some(TokenData::Equals) => {
                        let value = self.parse_expr()?;
                        Ok(Node::VarAssign(name, Box::new(value)))
                    }
                    _ => Err(ParseError("Expected '('".into())),
                }
            }
            TokenData::Let => {
                if let Some(TokenData::Name(name)) = self.nom() {
                    let name = name.clone();
                    self.expect(TokenData::Equals)?;

                    Ok(Node::VarDecl(name, Box::new(self.parse_expr()?)))
                } else {
                    Err(ParseError(line!().to_string()))
                }
            }
            _ => Err(ParseError(line!().to_string())),
        }
    }

    fn parse_expr(&mut self) -> ParseResult<Node> {
        match self.nom().ok_or(ParseError("Expected expression".into()))? {
            TokenData::StrLit(string) => Ok(Node::StrLit(string.clone())),
            TokenData::Name(name) => Ok(Node::VarAccess(name.clone())),
            TokenData::Int(int) => Ok(Node::Int(*int)),
            TokenData::Plus => {
                let a = self.parse_expr()?;
                let b = self.parse_expr()?;

                Ok(Node::BinOp(BinOp::Plus, Box::new(a), Box::new(b)))
            }
            TokenData::Minus => {
                let a = self.parse_expr()?;
                let b = self.parse_expr()?;

                Ok(Node::BinOp(BinOp::Minus, Box::new(a), Box::new(b)))
            }
            TokenData::Mult => {
                let a = self.parse_expr()?;
                let b = self.parse_expr()?;

                Ok(Node::BinOp(BinOp::Mult, Box::new(a), Box::new(b)))
            }
            TokenData::Div => {
                let a = self.parse_expr()?;
                let b = self.parse_expr()?;

                Ok(Node::BinOp(BinOp::Div, Box::new(a), Box::new(b)))
            }
            _ => Err(ParseError("Expected expression".into())),
        }
    }

    /*
    fn expect_fn<T>(&mut self, f: T) -> ParseResult<()>
        where T: FnOnce(&TokenData) -> bool {
        if f(self.nom().ok_or(ParseError(line!().to_string()))?) {
            Ok(())
        } else {
            Err(ParseError(line!().to_string()))
        }
    }*/

    fn is_empty(&self) -> bool {
        self.i >= self.lexer.len()
    }

    fn expect(&mut self, tok: TokenData) -> ParseResult<()> {
        if self.nom() == Some(&tok) {
            Ok(())
        } else {
            Err(ParseError(line!().to_string()))
        }
    }

    fn nom(&mut self) -> Option<&TokenData> {
        let tok = self.lexer.get(self.i)?;
        self.i += 1;
        Some(&tok.data)
    }
}

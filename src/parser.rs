use crate::lexer::{Token, TokenData};

pub type NodeList = Vec<Node>;

#[derive(Debug, Clone, Copy)]
pub enum BinOp {
    Plus,
    Minus,
    Mult,
    Div,
}

#[derive(Debug, Clone, Copy)]
pub enum CmpOp {
    Less,
    Equal,
    Greater,
    LtEq,
    GtEq,
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
    CmpOp(CmpOp, Box<Node>, Box<Node>),
    If {cond: Box<Node>, then_branch: Box<Node>},
}

// TODO: Provide details for parse error
#[derive(Debug)]
pub enum ParseError {
    Error(String),
    BlockEnding
}

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

use ParseError::Error;
impl<'a> Parser<'a> {
    fn parse_block(&mut self) -> ParseResult<Node> {
        println!("{:#?}", self.lexer);
        let mut statements = Vec::new();

        // let st = self.parse_statement()?;
        // statements.push(st);

        while !self.is_empty() {
            let res = self.parse_statement();
            if let Err(ParseError::BlockEnding) = res {
                break;
            }
            let st = res?;
            statements.push(st);
        }

        Ok(Node::Block(statements))
    }

    fn parse_statement(&mut self) -> ParseResult<Node> {
        let res = match self.nom().ok_or(Error(line!().to_string()))? {
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
                    _ => Err(Error("Expected '('".into())),
                }
            }
            TokenData::Let => {
                if let Some(TokenData::Name(name)) = self.nom() {
                    let name = name.clone();
                    self.expect(TokenData::Equals)?;

                    Ok(Node::VarDecl(name, Box::new(self.parse_expr()?)))
                } else {
                    Err(Error(line!().to_string()))
                }
            }
            TokenData::If => {
                let cond = self.parse_expr()?;

                self.expect(TokenData::LCurly)?;
                let then_branch = self.parse_block()?;
                // NOTE: We don't need this because parse_block() already handles the '}'
                //self.expect(TokenData::RCurly)?;

                // We use return to avoid handling semicolon
                return Ok(Node::If { cond: Box::new(cond), then_branch: Box::new(then_branch) });
            }
            TokenData::RCurly => Err(ParseError::BlockEnding),
            _ => Err(Error(line!().to_string())),
        }?;
        self.expect(TokenData::Semicolon)?;
        Ok(res)
    }

    fn parse_expr(&mut self) -> ParseResult<Node> {
        match self.nom().ok_or(Error("Expected expression".into()))? {
            TokenData::StrLit(string) => Ok(Node::StrLit(string.clone())),
            TokenData::Name(name) => Ok(Node::VarAccess(name.clone())),
            TokenData::Int(int) => Ok(Node::Int(*int)),

            TokenData::Plus => self.parse_bin_op(BinOp::Plus),
            TokenData::Minus => self.parse_bin_op(BinOp::Minus),
            TokenData::Mult => self.parse_bin_op(BinOp::Mult),
            TokenData::Div => self.parse_bin_op(BinOp::Div),

            TokenData::Less => self.parse_cmp_op(CmpOp::Less),
            TokenData::EqEq => self.parse_cmp_op(CmpOp::Equal),
            TokenData::Greater => self.parse_cmp_op(CmpOp::Greater),
            TokenData::LtEq => self.parse_cmp_op(CmpOp::LtEq),
            TokenData::GtEq => self.parse_cmp_op(CmpOp::GtEq),
            
            _ => Err(Error("Expected expression".into())),
        }
    }

    fn parse_bin_op(&mut self, op: BinOp) -> ParseResult<Node> {
        let a = self.parse_expr()?;
        let b = self.parse_expr()?;

        Ok(Node::BinOp(op, Box::new(a), Box::new(b)))
    }

    fn parse_cmp_op(&mut self, op: CmpOp) -> ParseResult<Node> {
        let a = self.parse_expr()?;
        let b = self.parse_expr()?;

        Ok(Node::CmpOp(op, Box::new(a), Box::new(b)))
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
            Err(Error(line!().to_string()))
        }
    }

    fn nom(&mut self) -> Option<&TokenData> {
        let tok = self.lexer.get(self.i)?;
        self.i += 1;
        Some(&tok.data)
    }
}

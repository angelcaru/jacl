use crate::{
    lexer::{Token, TokenData},
    loc::Loc,
};

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
    FuncCall(Loc, String, NodeList),
    StrLit(Loc, String),
    Block(Loc, NodeList),
    VarDecl(Loc, String, Box<Node>),
    VarAccess(Loc, String),
    VarAssign(Loc, String, Box<Node>),
    Int(Loc, usize),
    BinOp(Loc, BinOp, Box<Node>, Box<Node>),
    CmpOp(Loc, CmpOp, Box<Node>, Box<Node>),
    If {
        loc: Loc,
        cond: Box<Node>,
        then_branch: Box<Node>,
        else_branch: Option<Box<Node>>,
    },
    Nop(Loc),
    While {
        loc: Loc,
        cond: Box<Node>,
        body: Box<Node>,
    },
    FuncDef {
        loc: Loc,
        name: String,
        args: Vec<String>,
        body: Box<Node>,
    },
    Buf(Loc, usize),
    PtrAccess(Loc, Box<Node>),
    PtrAssign(Loc, Box<Node>, Box<Node>),
    VarAddr(Loc, String),
}

#[derive(Debug)]
pub enum ParseError {
    Error(Loc, String),
    LexerError(Loc, String),
    BlockEnding,
}

type ParseResult<T> = Result<T, ParseError>;

pub fn parse<T: Iterator<Item = Result<Token, (Loc, String)>>>(
    lexer: T,
    debug: bool,
) -> ParseResult<Node> {
    let mut tokens = Vec::new();
    for res in lexer {
        match res {
            Ok(tok) => tokens.push(tok),
            Err((loc, err)) => Err(ParseError::LexerError(loc, err))?,
        }
    }
    if debug {
        println!("{:#?}", tokens);
    }
    Parser {
        lexer: tokens,
        i: 0,
    }
    .parse_block()
}

struct Parser {
    lexer: Vec<Token>,
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
impl Parser {
    fn parse_block(&mut self) -> ParseResult<Node> {
        let loc = self.loc();
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

        Ok(Node::Block(loc, statements))
    }

    fn parse_statement(&mut self) -> ParseResult<Node> {
        let loc = self.loc();
        let res = match self.peek().expect("On EOF we shouldn't be here") {
            TokenData::Name(name) => {
                let name = name.clone();
                self.nom();

                match self.nom() {
                    Some(TokenData::LParen) => {
                        let mut args = vec![self.parse_expr()?];
                        while let Some(TokenData::Comma) = self.peek() {
                            self.nom();
                            args.push(self.parse_expr()?);
                        }
                        self.expect(TokenData::RParen)?;
                        Ok(Node::FuncCall(loc, name, args))
                    }
                    Some(TokenData::Equals) => {
                        let value = self.parse_expr()?;
                        Ok(Node::VarAssign(loc, name, Box::new(value)))
                    }
                    _ => Err(Error(loc, "Expected '(' or '='".into())),
                }
            }
            TokenData::Let => {
                self.nom();
                if let Some(TokenData::Name(name)) = self.nom() {
                    let name = name.clone();
                    self.expect(TokenData::Equals)?;

                    Ok(Node::VarDecl(loc, name, Box::new(self.parse_expr()?)))
                } else {
                    Err(Error(loc, "Expected identifier".into()))
                }
            }
            TokenData::If => {
                self.nom();
                let cond = self.parse_expr()?;

                self.expect(TokenData::LCurly)?;
                let then_branch = self.parse_block()?;
                // NOTE: We don't need this because parse_block() already handles the '}'
                //self.expect(TokenData::RCurly)?;

                if let Some(TokenData::Else) = self.peek() {
                    self.nom();

                    self.expect(TokenData::LCurly)?;
                    let else_branch = self.parse_block()?;

                    return Ok(Node::If {
                        loc,
                        cond: Box::new(cond),
                        then_branch: Box::new(then_branch),
                        else_branch: Some(Box::new(else_branch)),
                    });
                } else {
                    // We use return to avoid handling semicolon
                    return Ok(Node::If {
                        loc,
                        cond: Box::new(cond),
                        then_branch: Box::new(then_branch),
                        else_branch: None,
                    });
                }
            }
            TokenData::Unless => {
                self.nom();
                let cond = self.parse_expr()?;

                self.expect(TokenData::LCurly)?;
                let then_branch = self.parse_block()?;
                // NOTE: We don't need this because parse_block() already handles the '}'
                //self.expect(TokenData::RCurly)?;

                // We use return to avoid handling semicolon
                return Ok(Node::If {
                    loc: loc.clone(),
                    cond: Box::new(cond),
                    then_branch: Box::new(Node::Nop(loc)),
                    else_branch: Some(Box::new(then_branch)),
                });
            }
            TokenData::While => {
                self.nom();
                let cond = self.parse_expr()?;

                self.expect(TokenData::LCurly)?;
                let body = self.parse_block()?;

                return Ok(Node::While {
                    loc,
                    cond: Box::new(cond),
                    body: Box::new(body),
                });
            }
            TokenData::Fun => {
                self.nom();
                let name = self.parse_ident()?;
                self.expect(TokenData::LParen)?;

                let mut args = vec![self.parse_ident()?];
                while let Some(TokenData::Comma) = self.peek() {
                    self.nom();
                    args.push(self.parse_ident()?);
                }
                self.expect(TokenData::RParen)?;

                self.expect(TokenData::LCurly)?;
                let body = self.parse_block()?;

                return Ok(Node::FuncDef {
                    loc,
                    name,
                    args,
                    body: Box::new(body),
                });
            }
            TokenData::Bang => {
                self.nom();
                let ptr = self.parse_expr()?;
                self.expect(TokenData::Equals)?;

                let expr = self.parse_expr()?;

                Ok(Node::PtrAssign(loc, Box::new(ptr), Box::new(expr)))
            }
            TokenData::RCurly => {
                self.nom(); // Here it would make sense not to nom() but I don't want to rewrite everything
                Err(ParseError::BlockEnding)
            },
            _ => self.parse_expr(),
        }?;
        self.expect(TokenData::Semicolon)?;
        Ok(res)
    }

    fn parse_ident(&mut self) -> ParseResult<String> {
        if let Some(TokenData::Name(str)) = self.nom() {
            Ok(str.clone())
        } else {
            Err(Error(self.loc(), "Expected identifier".into()))
        }
    }

    fn parse_expr(&mut self) -> ParseResult<Node> {
        let loc = self.loc();
        match self
            .nom()
            .ok_or(Error(loc.clone(), "Expected expression".into()))?
        {
            TokenData::StrLit(string) => Ok(Node::StrLit(loc, string.clone())),
            TokenData::Name(name) => {
                let name = name.clone();

                match self.peek() {
                    Some(TokenData::LParen) => {
                        self.nom();
                        let mut args = vec![self.parse_expr()?];
                        while let Some(TokenData::Comma) = self.peek() {
                            self.nom();
                            args.push(self.parse_expr()?);
                        }
                        self.expect(TokenData::RParen)?;
                        Ok(Node::FuncCall(loc, name, args))
                    }
                    _ => Ok(Node::VarAccess(loc, name.clone())),
                }
            },
            TokenData::Int(int) => Ok(Node::Int(loc, *int)),

            TokenData::Plus => self.parse_bin_op(BinOp::Plus),
            TokenData::Minus => self.parse_bin_op(BinOp::Minus),
            TokenData::Mult => self.parse_bin_op(BinOp::Mult),
            TokenData::Div => self.parse_bin_op(BinOp::Div),

            TokenData::Less => self.parse_cmp_op(CmpOp::Less),
            TokenData::EqEq => self.parse_cmp_op(CmpOp::Equal),
            TokenData::Greater => self.parse_cmp_op(CmpOp::Greater),
            TokenData::LtEq => self.parse_cmp_op(CmpOp::LtEq),
            TokenData::GtEq => self.parse_cmp_op(CmpOp::GtEq),

            TokenData::Buf => {
                if let Some(TokenData::Int(size)) = self.nom() {
                    Ok(Node::Buf(loc, *size))
                } else {
                    Err(Error(loc, "Expected integer literal".into()))
                }
            }
            TokenData::Bang => {
                let expr = self.parse_expr()?;
                Ok(Node::PtrAccess(loc, Box::new(expr)))
            }
            TokenData::Amp => {
                let name = self.parse_ident()?;
                Ok(Node::VarAddr(loc, name))
            }

            _ => Err(Error(loc, "Expected expression".into())),
        }
    }

    fn parse_bin_op(&mut self, op: BinOp) -> ParseResult<Node> {
        let loc = self.loc();
        let a = self.parse_expr()?;
        let b = self.parse_expr()?;

        Ok(Node::BinOp(loc, op, Box::new(a), Box::new(b)))
    }

    fn parse_cmp_op(&mut self, op: CmpOp) -> ParseResult<Node> {
        let loc = self.loc();
        let a = self.parse_expr()?;
        let b = self.parse_expr()?;

        Ok(Node::CmpOp(loc, op, Box::new(a), Box::new(b)))
    }

    fn is_empty(&self) -> bool {
        self.i >= self.lexer.len()
    }

    fn expect(&mut self, tok: TokenData) -> ParseResult<()> {
        if self.nom() == Some(&tok) {
            Ok(())
        } else {
            Err(Error(self.loc(), format!("Expected {:?}", tok)))
        }
    }

    fn peek(&self) -> Option<&TokenData> {
        Some(&self.lexer.get(self.i)?.data)
    }

    fn nom(&mut self) -> Option<&TokenData> {
        let tok = self.lexer.get(self.i)?;
        self.i += 1;
        println!("{:?} nom nom", tok.data);
        Some(&tok.data)
    }

    fn loc(&self) -> Loc {
        let i = if self.i < self.lexer.len() {
            self.i
        } else {
            self.i - 1
        };
        self.lexer
            .get(i)
            .unwrap()
            .loc
            .clone()
    }
}

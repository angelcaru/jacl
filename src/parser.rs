use crate::lexer::{Token, TokenData};

pub type NodeList = Vec<Node>;

#[derive(Debug)]
pub enum Node {
    FuncCall(String, NodeList),
    StrLit(String),
    Block(NodeList),
}

// TODO: Provide details for parse error
#[derive(Debug)]
pub struct ParseError;

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
        let mut statements = Vec::new();

        if let Ok(st) = self.parse_statement() {
            statements.push(st);

            while let Some(TokenData::Semicolon) = self.nom() {
                let st = self.parse_statement();
                if st.is_err() {
                    break;
                }
                let st = st.unwrap();
                statements.push(st);
            }
        }

        Ok(Node::Block(statements))
    }

    fn parse_statement(&mut self) -> ParseResult<Node> {
        if let Some(TokenData::Name(name)) = self.nom() {
            let name = name.clone();

            self.expect(TokenData::LParen)?;
            if let Some(TokenData::StrLit(string)) = self.nom() {
                let string = string.clone();
                self.expect(TokenData::RParen)?;
                Ok(Node::FuncCall(name, vec![Node::StrLit(string)]))
            } else {
                Err(ParseError)
            }
        } else {
            Err(ParseError)
        }
    }

    /*
    fn expect_fn<T>(&mut self, f: T) -> ParseResult<()>
        where T: FnOnce(&TokenData) -> bool {
        if f(self.nom().ok_or(ParseError)?) {
            Ok(())
        } else {
            Err(ParseError)
        }
    }*/

    fn expect(&mut self, tok: TokenData) -> ParseResult<()> {
        if self.nom() == Some(&tok) {
            Ok(())
        } else {
            Err(ParseError)
        }
    }

    fn nom(&mut self) -> Option<&TokenData> {
        let tok = self.lexer.get(self.i)?;
        self.i += 1;
        Some(&tok.data)
    }
}

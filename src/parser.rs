use crate::{
    bug, error,
    lexer::{TType, Token},
    OrionError, Result,
};
use std::mem::discriminant;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Var(String),
    Call(Box<Expr>, Box<Expr>),
    Lambda(String, Box<Expr>),
    Integer(i32),
    Single(f32),
    Boolean(bool),
    Def(String, Box<Expr>),
    Enum(String, Vec<String>, Vec<u8>),
    Unit,
    String(String),
}

pub struct Parser {
    input: Vec<Token>,
    output: Vec<Expr>,
    current: usize,
}

impl Parser {
    pub fn new(input: Vec<Token>) -> Self {
        Self {
            input,
            output: vec![],
            current: 0usize,
        }
    }
    fn advance(&mut self, expected: TType) -> Result<Token> {
        let popped = self.pop()?;

        if discriminant(&popped.ttype) != discriminant(&expected) {
            error!(
                "{}:{} | Expected {}, found {}.",
                popped.line,
                popped.col,
                expected.get_type(),
                popped.ttype.get_type()
                )
        } else {
            Ok(popped)
        }
    }
    fn pop(&mut self) -> Result<Token> {
        if self.current + 1 >= self.input.len() {
            let previous = &self.input[self.current];
            error!(
                "{}:{} | Unfinished expression.",
                previous.line, previous.col
                )
        } else {
            self.current += 1;
            Ok(self.input[self.current - 1].clone())
        }
    }
    fn peek(&self) -> Option<Token> {
        self.input
            .iter()
            .nth(self.current)
            .and_then(|t| Some(t.clone()))
    }
    fn is_at_end(&self) -> bool {
        self.current + 1 >= self.input.len()
    }
    fn advance_many(&mut self, expected: TType) -> Result<Vec<Token>> {
        let mut toret = vec![];

        while !self.is_at_end()
            && std::mem::discriminant(&self.peek().unwrap().ttype) == discriminant(&expected)
            {
                toret.push(self.advance(expected.clone())?);
            }

        Ok(toret)
    }
    fn parse_expr(&mut self) -> Result<Expr> {
        let root = self.pop()?;

        Ok(match &root.ttype {
            TType::Str(s) => Expr::String(s.to_string()),
            TType::Float(f) => Expr::Single(*f),
            TType::Number(i) => Expr::Integer(*i),
            TType::Ident(v) => Expr::Var(v.to_string()),
            TType::LParen => {
                let subroot = self.pop()?;

                match &subroot.ttype {
                    TType::LParen => self.parse_expr()?,
                    TType::Def => {
                        let raw_name = self.advance(TType::Ident("".to_owned()))?;
                        let name = if let TType::Ident(n) = raw_name.ttype {
                            n
                        } else {
                            bug!("UNEXPECTED_NON_IDENTIFIER");
                        };

                        let value = self.parse_expr()?;

                        if !self.is_at_end() {
                            self.advance(TType::RParen)?;
                        }

                        Expr::Def(name, Box::new(value))
                    }
                    TType::Enum => {
                        let r_name = self.advance(TType::Ident("".to_owned()))?;

                        let name = if let TType::Ident(n) = r_name.ttype {
                            n
                        } else {
                            bug!("UNEXPECTED_NON_IDENTIFIER");
                        };

                        if !(65..91).contains(&(name.chars().nth(0).unwrap() as u8)) {
                            return error!("{}:{} | Enum names have to start with a capital letter.", r_name.line, r_name.col);
                        }

                        let mut variants = vec![];
                        let mut containing = vec![];

                        while !self.is_at_end() && self.peek().unwrap().ttype != TType::RParen {

                            self.advance(TType::LParen)?;

                            let r_name = self.advance(TType::Ident("".to_owned()))?;

                            let vname = if let TType::Ident(n) = r_name.ttype {
                                n
                            } else {
                                bug!("UNEXPECTED_NON_IDENTIFIER");
                            };


                            if !(65..91).contains(&(vname.chars().nth(0).unwrap() as u8)) {
                                return error!("{}:{} | Enum variant names have to start with a capital letter.", r_name.line, r_name.col);
                            }

                            let length = self.advance_many(TType::Ident("".to_owned()))?.len() as u8;

                            variants.push(vname);
                            containing.push(length);

                            self.advance(TType::RParen)?;
                        }

                        if !self.is_at_end() {
                            self.advance(TType::RParen)?;
                        }


                        Expr::Enum(name, variants, containing)
                    }
                    TType::Lambda => {
                        self.advance(TType::LParen)?;
                        let args = self.advance_many(TType::Ident("".to_owned()))?;
                        self.advance(TType::RParen)?;

                        let args = args.iter().map(|e| if let TType::Ident(ident) = &e.ttype {
                            ident
                        } else {
                            bug!("What is this thing doing here ?");
                        }).collect::<Vec<_>>();

                        let mut body = self.parse_expr()?;

                        for arg in args.into_iter().rev() {
                            body = Expr::Lambda(arg.to_string(), Box::new(body));
                        }
                        if !self.is_at_end() {
                            self.advance(TType::RParen)?;
                        }
                        body
                    }
                    TType::Ident(x) => {
                        let func = Expr::Var(x.to_string());
                        let mut args = vec![];
                        while !self.is_at_end() && self.peek().unwrap().ttype != TType::RParen {
                            args.push(self.parse_expr()?);
                        }

                        if !self.is_at_end() {
                            self.advance(TType::RParen)?;
                        }

                        let args = args.into_iter().fold(func, |root, elem| Expr::Call(Box::new(root), Box::new(elem)));
                        args
                    }
                    TType::RParen => Expr::Unit,
                    _ => return error!("{}:{} | Expected Closing Parenthese, Opening Parenthese or Identifier, found {}.", subroot.line, subroot.col, subroot.ttype.get_type()),
                }
            }
            TType::RParen => {
                return error!(
                    "{}:{} | Unexpected Closing Parenthese.",
                    root.line, root.col
                    )
            }
            _ => return error!("{}:{} | Unexpected Keyword.", root.line, root.col)
        })
    }

    pub fn parse(&mut self) -> Result<Vec<Expr>> {
        while !self.is_at_end() {
            let to_push = self.parse_expr()?;
            self.output.push(to_push);
        }

        Ok(self.output.clone())
    }
}

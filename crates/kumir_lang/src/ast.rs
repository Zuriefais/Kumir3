use log::info;

use crate::lexer::{
    self, Condition, Delimiter, Keyword, Loop, Operator, Range, Token, TypeDefinition,
};

#[derive(Debug, PartialEq, Clone)]
pub enum AstNode {
    Program(Vec<Stmt>),
    Stmt(Stmt),
    Expr(Expr),
}

#[derive(Debug, PartialEq, Clone)]
pub struct VarDecl {
    name: String,
    type_def: TypeDefinition,
    value: Option<Expr>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
    VarDecl(VarDecl),
    VarsDecl(Vec<VarDecl>),
    Assign {
        name: String,
        value: Expr,
    },
    Alg {
        name: String,
        body: Vec<Stmt>,
    },
    Condition {
        condition: Expr,
        left: Vec<Stmt>,
        right: Option<Vec<Stmt>>,
    },
    Loop {
        condition: Option<Expr>,
        body: Vec<Stmt>,
    },
    ForLoop {
        var: String,
        start: Expr,
        end: Expr,
        body: Vec<Stmt>,
    },
    Repeat {
        condition: Option<Expr>,
        count: Expr,
        body: Vec<Stmt>,
    },
    Break,
    Start,
    Stop,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Literal(Literal),
    Identifier(String),
    BinaryOp {
        left: Box<Expr>,
        op: Operator,
        right: Box<Expr>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Int(i32),
    Float(f32),
    String(String),
    Char(char),
    Bool(bool),
}

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            position: 0,
        }
    }

    pub fn parse(&mut self) -> Result<AstNode, String> {
        let mut statements = Vec::new();
        while !self.is_eof() {
            statements.push(self.parse_stmt()?);
        }
        Ok(AstNode::Program(statements))
    }

    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        match self.current_token().clone() {
            Token::Keyword(Keyword::Alg) => self.parse_alg(),
            Token::Keyword(Keyword::Start) => {
                self.advance();
                Ok(Stmt::Start)
            }
            Token::Keyword(Keyword::Condition(Condition::If)) => self.parse_condition(),
            Token::Keyword(Keyword::Loop(Loop::Start)) => self.parse_loop(),
            Token::Keyword(Keyword::Loop(Loop::Break)) => Ok(Stmt::Break),
            Token::Keyword(Keyword::Stop) => {
                self.advance();
                Ok(Stmt::Stop)
            }
            Token::Keyword(Keyword::TypeDef(type_def)) => self.parse_var_decl(&type_def),
            Token::Identifier(_) => self.parse_assign(),
            _ => Err(format!("Unexpected token: {:?}", self.current_token())),
        }
    }

    fn parse_condition(&mut self) -> Result<Stmt, String> {
        self.advance();
        todo!()
    }

    fn parse_loop(&mut self) -> Result<Stmt, String> {
        self.advance();
        match self.current_token() {
            Token::Keyword(Keyword::Loop(Loop::While)) => self.parse_while_loop(),
            Token::Int(times) => self.parse_repeat_loop(Expr::Literal(Literal::Int(*times))),
            Token::Keyword(Keyword::Range(Range::For)) => self.parse_for_loop(),

            _ => self.parse_simple_loop(),
        }
    }

    fn parse_simple_loop(&mut self) -> Result<Stmt, String> {
        self.advance();
        let mut statements = Vec::new();
        let mut condition = None;
        while *self.current_token() != Token::Keyword(Keyword::Loop(Loop::End)) {
            if *self.current_token() != Token::Keyword(Keyword::Loop(Loop::EndIf)) {
                self.advance();
                condition = Some(self.parse_expr()?);
            } else {
                statements.push(self.parse_stmt()?);
            }
        }
        Ok(Stmt::Loop {
            condition,
            body: statements,
        })
    }

    fn parse_while_loop(&mut self) -> Result<Stmt, String> {
        self.advance();
        let condition = Some(self.parse_expr()?);
        let mut statements = Vec::new();
        while *self.current_token() != Token::Keyword(Keyword::Loop(Loop::End)) {
            statements.push(self.parse_stmt()?);
        }
        self.advance();
        Ok(Stmt::Loop {
            condition,
            body: statements,
        })
    }

    fn parse_for_loop(&mut self) -> Result<Stmt, String> {
        self.advance();
        let var = match self.current_token() {
            Token::Identifier(name) => name.clone(),
            _ => return Err("Expected identifier".to_string()),
        };
        self.advance();
        self.expect(Token::Keyword(Keyword::Range(Range::From)))?;
        let start = self.parse_expr()?;
        self.expect(Token::Keyword(Keyword::Range(Range::To)))?;
        let end = self.parse_expr()?;
        let mut statements = Vec::new();
        while *self.current_token() != Token::Keyword(Keyword::Loop(Loop::End)) {
            statements.push(self.parse_stmt()?);
        }
        self.advance();
        Ok(Stmt::ForLoop {
            var,
            start,
            end,
            body: statements,
        })
    }

    fn parse_repeat_loop(&mut self, count: Expr) -> Result<Stmt, String> {
        self.advance();
        self.expect(Token::Keyword(Keyword::Loop(Loop::Times)))?;
        let mut statements = Vec::new();
        let mut condition = None;
        while *self.current_token() != Token::Keyword(Keyword::Loop(Loop::End)) {
            if *self.current_token() != Token::Keyword(Keyword::Loop(Loop::EndIf)) {
                self.advance();
                condition = Some(self.parse_expr()?);
            } else {
                statements.push(self.parse_stmt()?);
            }
        }
        self.advance();
        Ok(Stmt::Repeat {
            condition,
            count,
            body: statements,
        })
    }

    fn parse_alg(&mut self) -> Result<Stmt, String> {
        self.advance(); // Skip Alg
        let name = match self.current_token().clone() {
            Token::Identifier(name) => {
                self.advance();
                name.clone()
            }
            _ => format!("main"),
        };

        // self.expect(Token::Delimiter(Delimiter::ParenthesisOpen))?;
        // self.expect(Token::Delimiter(Delimiter::ParenthesisClose))?;
        let mut body = Vec::new();
        while !self.check(&Token::Keyword(Keyword::Stop)) && !self.is_eof() {
            body.push(self.parse_stmt()?);
        }
        self.expect(Token::Keyword(Keyword::Stop))?;
        Ok(Stmt::Alg { name, body })
    }

    fn parse_var_decl(&mut self, type_def: &TypeDefinition) -> Result<Stmt, String> {
        self.advance(); // Skip type
        let name = match self.current_token() {
            Token::Identifier(name) => name.clone(),
            _ => return Err("Expected identifier".to_string()),
        };
        info!("Creating var with name: {}", &name);
        self.advance();

        match self.current_token() {
            Token::Operator(lexer::Operator::Assignment) => {
                self.advance();
                let value = Some(self.parse_expr()?);
                Ok(Stmt::VarDecl(VarDecl {
                    name,
                    type_def: type_def.clone(),
                    value,
                }))
            }
            Token::Delimiter(Delimiter::Comma) => {
                let mut vars = vec![VarDecl {
                    name: name.clone(),
                    type_def: type_def.clone(),
                    value: None,
                }];
                while self.check(&Token::Delimiter(Delimiter::Comma)) {
                    self.advance();
                    match self.current_token() {
                        Token::Identifier(name) => {
                            vars.push(VarDecl {
                                name: name.clone(),
                                type_def: type_def.clone(),
                                value: None,
                            });
                        }
                        _ => return Err(format!("Couldn't construct a vars sequence")),
                    };
                    self.advance();
                }
                Ok(Stmt::VarsDecl(vars))
            }
            _ => Ok(Stmt::VarDecl(VarDecl {
                name,
                type_def: type_def.clone(),
                value: None,
            })),
        }
    }

    fn parse_assign(&mut self) -> Result<Stmt, String> {
        let name = match self.current_token() {
            Token::Identifier(name) => name.clone(),
            _ => return Err("Expected identifier".to_string()),
        };
        self.advance();
        self.expect(Token::Operator(lexer::Operator::Assignment))?;
        let value = self.parse_expr()?;
        Ok(Stmt::Assign { name, value })
    }

    fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_binary_expr(0)
    }

    fn parse_binary_expr(&mut self, precedence: i32) -> Result<Expr, String> {
        let mut left = self.parse_primary()?;
        while let Some(op) = self.current_token().as_operator() {
            let op_precedence = op.precedence();
            if op_precedence < precedence {
                break;
            }
            self.advance();
            let right = self.parse_binary_expr(op_precedence + 1)?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        match self.current_token().clone() {
            Token::Int(i) => {
                self.advance();
                Ok(Expr::Literal(Literal::Int(i)))
            }
            Token::Float(f) => {
                self.advance();
                Ok(Expr::Literal(Literal::Float(f)))
            }
            Token::String(s) => {
                self.advance();
                Ok(Expr::Literal(Literal::String(s.clone())))
            }
            Token::Char(c) => {
                self.advance();
                Ok(Expr::Literal(Literal::Char(c)))
            }
            Token::Bool(b) => {
                self.advance();
                Ok(Expr::Literal(Literal::Bool(b)))
            }
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                Ok(Expr::Identifier(name))
            }
            Token::Delimiter(Delimiter::ParenthesisOpen) => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(Token::Delimiter(Delimiter::ParenthesisClose))?;
                Ok(expr)
            }
            _ => Err(format!(
                "Unexpected token in expression: {:?}",
                self.current_token()
            )),
        }
    }

    fn current_token(&self) -> &Token {
        self.tokens.get(self.position).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) {
        self.position += 1;
    }

    fn check(&self, token: &Token) -> bool {
        self.current_token() == token
    }

    fn expect(&mut self, token: Token) -> Result<(), String> {
        if self.check(&token) {
            self.advance();
            Ok(())
        } else {
            Err(format!(
                "Expected {:?}, found {:?}, current token: {}",
                token,
                self.current_token(),
                self.position
            ))
        }
    }

    fn is_eof(&self) -> bool {
        self.current_token() == &Token::Eof
    }
}

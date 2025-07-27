use log::info;

use crate::{
    ast::{AstNode, Environment, Parser},
    lexer::{Lexer, Token},
};

pub struct Interpreter {
    pub ast: AstNode,
    pub environment: Environment,
}

impl Interpreter {
    pub fn run(&mut self) -> Result<(), String> {
        match self.ast.eval(&mut self.environment) {
            Ok(()) => {
                info!("Все ок!!");
                Ok(())
            }
            Err(e) => Err(format!("Ошибка выполнения: {e}")),
        }
    }

    pub fn new(ast: AstNode) -> Self {
        Interpreter {
            ast,
            environment: Environment::new(),
        }
    }

    pub fn new_from_tokens(tokens: Vec<Token>) -> Result<Self, String> {
        Ok(Self::new(Parser::new(tokens).parse()?))
    }

    pub fn new_from_string(input: &str) -> Result<Self, String> {
        let mut lexer = Lexer::new(input);
        let mut tokens = vec![];
        loop {
            match lexer.next_token() {
                Ok(Token::Eof) => break,
                Ok(token) => {
                    tokens.push(token);
                }
                Err(e) => println!("Ошибка: {e}"),
            }
        }
        Self::new_from_tokens(tokens)
    }
}

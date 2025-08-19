use log::info;

use crate::{
    ast::{AstNode, Environment, Parser, Stmt},
    lexer::{Lexer, Token},
};

pub struct Interpreter {
    pub ast: AstNode,
    pub environment: Environment,
}

impl Interpreter {
    pub fn run(&mut self) -> Result<(), String> {
        self.register_functions();
        match self.ast.eval(&mut self.environment) {
            Ok(_) => {
                info!("Program finished successfully");
                Ok(())
            }
            Err(e) => Err(format!("Runtime error: {e}")),
        }
    }

    pub fn register_functions(&mut self) {
        if let AstNode::Program(body) = &self.ast {
            for stmt in body {
                if let Stmt::Alg(alg) = stmt {
                    self.environment.register_function(
                        &alg.name,
                        crate::ast::FunctionVariant::Kumir(alg.clone()),
                    );
                }
            }
        }
    }

    pub fn new(ast: AstNode) -> Self {
        Interpreter {
            ast,
            environment: Environment::new(),
        }
    }

    // pub fn new_from_tokens(tokens: Vec<Token>) -> Result<Self, String> {
    //     Ok(Self::new(Parser::new(tokens).parse()?))
    // }

    // pub fn new_from_string(input: &str) -> Result<Self, String> {
    //     let mut lexer = Lexer::new(input);
    //     let mut tokens = vec![];
    //     loop {
    //         match lexer.next_token() {
    //             Ok(Token::Eof) => break,
    //             Ok(token) => {
    //                 tokens.push(token);
    //             }
    //             Err(e) => println!("Ошибка: {e}"),
    //         }
    //     }
    //     Self::new_from_tokens(tokens)
    // }
}

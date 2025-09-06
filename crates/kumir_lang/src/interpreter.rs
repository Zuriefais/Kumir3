use std::{cell::RefCell, rc::Rc};

use hashbrown::HashMap;
use log::{error, info};

use crate::{
    ast::{AstNode, Environment, FunctionVariant, Namespace, NativeFunction, Stmt},
    lexer::{Lexer, Token},
    parser::Parser,
};

pub struct Interpreter {
    pub ast: AstNode,
    pub environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn run(&mut self) -> Result<(), String> {
        self.register_functions();
        match self.ast.eval(&self.environment) {
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
                    self.environment.borrow_mut().register_function(
                        &alg.name,
                        crate::ast::FunctionVariant::Kumir(alg.clone()),
                    );
                }
            }
        }
    }

    pub fn register_native_function(&mut self, name: &str, function: NativeFunction) {
        self.environment
            .borrow_mut()
            .register_function(name, crate::ast::FunctionVariant::Native(function));
    }

    pub fn register_native_function_in_namespace(
        &mut self,
        name: &str,
        function: NativeFunction,
        namespace: &str,
    ) {
        self.environment
            .borrow_mut()
            .register_function_in_namespace(
                namespace,
                name,
                crate::ast::FunctionVariant::Native(function),
            );
    }

    pub fn register_namespace(&mut self, name: &str, namespace: Namespace) {
        self.environment
            .borrow_mut()
            .register_namespace(name, namespace);
    }

    pub fn new(ast: AstNode) -> Self {
        Interpreter {
            ast,
            environment: Default::default(),
        }
    }

    pub fn new_from_tokens(tokens: Vec<Token>) -> Result<Self, String> {
        let mut parser = Parser::new(tokens);
        match parser.parse() {
            Ok(ast) => {
                info!("AST generated: {ast:#?}");
                let interpreter = Interpreter::new(ast);
                return Ok(interpreter);
            }
            Err(err) => {
                let (statements, err) = err;
                error!("Error parsing AST: {}", err);
                error!("Statements parsed: {:#?}", statements);
                error!("AST generator stopped at token: {}", parser.position);
                return Err(format!("Error creating interpreter"));
            }
        }
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
                Err(e) => {
                    info!(
                        "Error: {e}, tokens parsed {:#?}",
                        tokens.iter().enumerate().collect::<Vec<_>>()
                    );
                }
            }
        }
        info!(
            "tokens parsed: {:#?}",
            tokens.iter().enumerate().collect::<Vec<_>>()
        );
        Self::new_from_tokens(tokens)
    }
}

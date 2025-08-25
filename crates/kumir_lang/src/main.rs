use std::{cell::RefCell, rc::Rc};

use indexmap::IndexMap;
use kumir_lang::{
    ast::{Environment, FunctionParameter, Literal, NativeFunction},
    interpreter::Interpreter,
    lexer::{self, Lexer, Token, TypeDefinition},
    parser::Parser,
};
use log::info;

fn main() {
    env_logger::init();
    let code = include_str!("test3.kum");
    let mut lexer = Lexer::new(code);
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
    info!(
        "Tokens:\n{}",
        tokens
            .iter()
            .enumerate()
            .map(|(i, t)| format!("  {i}: {t:#?}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    let mut parser = Parser::new(tokens);
    match parser.parse() {
        Ok(ast) => {
            info!("AST generated: {ast:#?}");
            let mut interpreter = Interpreter::new(ast);
            interpreter.register_native_function(
                "раст",
                NativeFunction {
                    params: {
                        let mut params = IndexMap::new();
                        params.insert("число".to_string(), FunctionParameter { type_definition: TypeDefinition::Float, result_type: lexer::FunctionParamType::ArgumentParam });
                        params
                    },
                    return_type: Some(TypeDefinition::Float),
                    native_function: Rc::new(RefCell::new(|environment: &Rc<RefCell<Environment>>| -> Result<Option<kumir_lang::ast::Literal>, String> {let num = environment.borrow().get_value("число").unwrap(); if let Literal::Float(num) = num {return Ok(Some(Literal::Float(num*40.0)))} return Err(format!("Err")); })),
                },
            );

            info!("Interpreter result: {:#?}", interpreter.run());

            info!("Interpreter environment: {:#?}", interpreter.environment);
        }
        Err(err) => {
            let (statements, err) = err;
            info!("Error parsing AST: {}", err);
            info!("Statements parsed: {:#?}", statements);
            info!("AST generator stopped at token: {}", parser.position);
        }
    }
}

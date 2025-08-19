use indexmap::IndexMap;
use kumir_lang::{
    ast::{
        self, AstNode, Environment, Expr, FunctionParameter, Literal, NativeFunction, Parser, Stmt,
        VarDecl,
    },
    interpreter::Interpreter,
    lexer::{self, Lexer, Token, TypeDefinition},
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
                    native_function: |environment: &mut Environment| -> Result<Option<kumir_lang::ast::Literal>, String> {let num = environment.get_value("число").unwrap(); if let Literal::Float(num) = num {return Ok(Some(Literal::Float(num*40.0)))} return Err(format!("Err")); },
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

pub fn create_test_ast() -> AstNode {
    let var_decl = Stmt::VarDecl(VarDecl {
        name: "x".to_string(),
        type_def: lexer::TypeDefinition::Int,
        value: Some(Expr::Literal(Literal::Int(42))),
    });

    let assign = Stmt::Assign {
        name: "x".to_string(),
        value: Expr::BinaryOp(ast::BinaryOp {
            left: Box::new(Expr::Identifier("x".to_string())),
            op: crate::lexer::Operator::Plus,
            right: Box::new(Expr::Literal(Literal::Int(1))),
        }),
    };

    let condition = Stmt::Condition {
        condition: Expr::BinaryOp(ast::BinaryOp {
            left: Box::new(Expr::Identifier("x".to_string())),
            op: crate::lexer::Operator::Less,
            right: Box::new(Expr::Literal(Literal::Int(50))),
        }),
        left: vec![Stmt::Assign {
            name: "x".to_string(),
            value: Expr::Literal(Literal::Int(100)),
        }],
        right: Some(vec![Stmt::Assign {
            name: "x".to_string(),
            value: Expr::Literal(Literal::Int(0)),
        }]),
    };

    AstNode::Program(vec![var_decl, assign, condition])
}

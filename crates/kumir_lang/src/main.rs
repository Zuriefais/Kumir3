use kumir_lang::{
    ast::{self, AstNode, Expr, Literal, Parser, Stmt, VarDecl},
    interpreter::{self, Interpreter},
    lexer::{self, Lexer, Token},
};
use log::info;

fn main() {
    env_logger::init();
    let code = include_str!("test2.kum");
    let mut lexer = Lexer::new(code);
    let mut tokens = vec![];
    loop {
        match lexer.next_token() {
            Ok(Token::Eof) => break,
            Ok(token) => {
                tokens.push(token);
            }
            Err(e) => println!("Ошибка: {}", e),
        }
    }
    info!(
        "Tokens:\n{}",
        tokens
            .iter()
            .enumerate()
            .map(|(i, t)| format!("  {}: {:#?}", i, t))
            .collect::<Vec<_>>()
            .join("\n")
    );
    let mut parser = Parser::new(tokens);
    let ast = parser.parse();
    info!("AST generated: {:#?}", ast);

    let mut interpreter = Interpreter::new(create_test_ast());

    info!("Interpreter result: {:?}", interpreter.run());

    info!("Interpreter environment: {:?}", interpreter.environment);
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

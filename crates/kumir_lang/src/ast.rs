use crate::lexer::{Token, TypeDefinition};

#[derive(Debug, PartialEq)]
pub enum AstNode {
    Program(Vec<Stmt>), // Корень AST - список операторов
    Stmt(Stmt),         // Оператор
    Expr(Expr),         // Выражение
}

#[derive(Debug, PartialEq)]
pub enum Stmt {
    AlgorithmDecl {
        name: String,    // Имя алгоритма (Identifier)
        body: Vec<Stmt>, // Тело алгоритма
    },
    Start, // Начало программы
    Stop,  // Конец программы
    VarDecl {
        name: String,             // Имя переменной (Identifier)
        type_def: TypeDefinition, // Тип (Int, Bool, Float, String, Char)
        init: Option<Expr>,       // Начальное значение (если есть)
    },
    Assign {
        name: String, // Имя переменной
        value: Expr,  // Присваиваемое выражение
    },
    ExprStmt(Expr), // Выражение как оператор
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Literal(Literal),   // Литералы
    Identifier(String), // Переменная или имя
    Binary {
        left: Box<Expr>,  // Левое подвыражение
        op: String,       // Оператор (+, -, *, /, и т.д.)
        right: Box<Expr>, // Правое подвыражение
    },
    Unary {
        op: String,      // Унарный оператор (-, !)
        expr: Box<Expr>, // Подвыражение
    },
}

#[derive(Debug, PartialEq)]
pub enum Literal {
    Int(i32),
    Float(f32),
    String(String),
    Char(char),
    Bool(bool),
}

struct ASTGenerator {
    input: Vec<Token>,
    position: usize,
    AstNode: AstNode,
}

impl ASTGenerator {
    pub fn new(tokens: &[Token]) -> ASTGenerator {
        let ast = AstNode::Program(vec![]);
        let generator = ASTGenerator {
            input: tokens.to_vec(),
            position: 0,
            AstNode: ast,
        };
        let mut statements = vec![];
        for i in 0..tokens.len() - 1 {
            match tokens[i] {
                Token::Keyword(keyword) => match keyword {
                    crate::lexer::Keyword::Alg => todo!(),
                    crate::lexer::Keyword::Start => {
                        statements.push(Stmt::Start);
                    }
                    crate::lexer::Keyword::Stop => {
                        statements.push(Stmt::Stop);
                    }
                    crate::lexer::Keyword::TypeDef(type_definition) => match type_definition {
                        TypeDefinition::Int => todo!(),
                        TypeDefinition::Bool => todo!(),
                        TypeDefinition::Float => todo!(),
                        TypeDefinition::String => todo!(),
                        TypeDefinition::Char => todo!(),
                    },
                },
                Token::Identifier(_) => todo!(),
                Token::Int(_) => todo!(),
                Token::Float(_) => todo!(),
                Token::Operator(_) => todo!(),
                Token::Delimiter(_) => todo!(),
                Token::String(_) => todo!(),
                Token::Char(_) => todo!(),
                Token::Bool(_) => todo!(),
                Token::Eof => todo!(),
            }
        }
        todo!()
    }
}

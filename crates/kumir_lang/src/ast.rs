use std::fmt::Display;

use hashbrown::HashMap;
use log::info;

use crate::lexer::{
    self, Condition, Delimiter, IO, Keyword, Loop, Operator, Range, Token, TypeDefinition,
};

#[derive(Debug, PartialEq, Clone)]
pub enum AstNode {
    Program(Vec<Stmt>),
    Stmt(Stmt),
    Expr(Expr),
}

#[derive(Debug, PartialEq, Clone)]
pub struct VarDecl {
    pub name: String,
    pub type_def: TypeDefinition,
    pub value: Option<Expr>,
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
    Output {
        values: Vec<Expr>,
    },
    Break,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Literal(Literal),
    Identifier(String),
    BinaryOp(BinaryOp),
    FunctionCall(FunctionCall),
    NewLine,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub args: Vec<Expr>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct BinaryOp {
    pub left: Box<Expr>,
    pub op: Operator,
    pub right: Box<Expr>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Int(i32),
    Float(f32),
    String(String),
    Char(char),
    Bool(bool),
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Int(value) => write!(f, "{}", value),
            Literal::Float(value) => write!(f, "{}", value),
            Literal::String(value) => write!(f, "\"{}\"", value),
            Literal::Char(value) => write!(f, "'{}'", value),
            Literal::Bool(value) => write!(f, "{}", value),
        }
    }
}

impl Literal {
    pub fn get_type(&self) -> TypeDefinition {
        match self {
            Literal::Int(_) => TypeDefinition::Int,
            Literal::Float(_) => TypeDefinition::Float,
            Literal::String(_) => TypeDefinition::String,
            Literal::Char(_) => TypeDefinition::Char,
            Literal::Bool(_) => TypeDefinition::Bool,
        }
    }
}

pub struct Parser {
    pub tokens: Vec<Token>,
    pub position: usize,
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
            Token::Keyword(Keyword::Condition(Condition::If)) => self.parse_condition(),
            Token::Keyword(Keyword::Loop(Loop::Start)) => self.parse_loop(),
            Token::Keyword(Keyword::Loop(Loop::Break)) => Ok(Stmt::Break),
            Token::Keyword(Keyword::TypeDef(type_def)) => self.parse_var_decl(&type_def),
            Token::Identifier(_) => self.parse_assign(),
            Token::Keyword(Keyword::IO(io_keyword)) => self.parse_io(io_keyword),
            _ => Err(format!("Unexpected token: {:?}", self.current_token())),
        }
    }

    fn parse_io(&mut self, io_keyword: IO) -> Result<Stmt, String> {
        match io_keyword {
            IO::Input => self.parse_input(),
            IO::Output => self.parse_output(),
            IO::ChangeLine => return Err(format!("Change line couldn't be Statement")),
        }
    }

    fn parse_input(&mut self) -> Result<Stmt, String> {
        self.advance();

        todo!()
    }

    fn parse_output(&mut self) -> Result<Stmt, String> {
        self.advance();
        let mut values = Vec::new();

        while !self.is_eof() {
            match self.current_token().clone() {
                Token::Keyword(Keyword::IO(IO::ChangeLine)) => {
                    values.push(Expr::NewLine);
                    self.advance();
                }
                Token::Delimiter(Delimiter::Comma) => {
                    self.advance();
                    continue;
                }
                Token::Keyword(_) => break,
                _ => {
                    let expr = self.parse_expr()?;
                    values.push(expr);
                }
            }
        }

        Ok(Stmt::Output { values })
    }

    fn parse_condition(&mut self) -> Result<Stmt, String> {
        self.advance();
        let condition = self.parse_expr()?;
        self.expect(Token::Keyword(Keyword::Condition(Condition::Then)))?;

        let mut left = Vec::new();
        while !self.check(&Token::Keyword(Keyword::Condition(Condition::Else)))
            && !self.check(&Token::Keyword(Keyword::Condition(Condition::EndCondition)))
        {
            left.push(self.parse_stmt()?);
        }

        let right = if self.check(&Token::Keyword(Keyword::Condition(Condition::Else))) {
            self.advance();
            let mut else_branch = Vec::new();
            while !self.check(&Token::Keyword(Keyword::Condition(Condition::EndCondition))) {
                else_branch.push(self.parse_stmt()?);
            }
            Some(else_branch)
        } else {
            None
        };

        self.expect(Token::Keyword(Keyword::Condition(Condition::EndCondition)))?;
        Ok(Stmt::Condition {
            condition,
            left,
            right,
        })
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
        self.advance(); // Skip the count
        self.expect(Token::Keyword(Keyword::Loop(Loop::Times)))?;
        let mut statements = Vec::new();
        while !self.check(&Token::Keyword(Keyword::Loop(Loop::End))) {
            statements.push(self.parse_stmt()?); // Parse statements, not expressions
        }
        self.advance(); // Skip Loop::End
        Ok(Stmt::Repeat {
            condition: None,
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
            _ => "main".to_string(),
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
                    type_def: *type_def,
                    value,
                }))
            }
            Token::Delimiter(Delimiter::Comma) => {
                let mut vars = vec![VarDecl {
                    name: name.clone(),
                    type_def: *type_def,
                    value: None,
                }];
                while self.check(&Token::Delimiter(Delimiter::Comma)) {
                    self.advance();
                    match self.current_token() {
                        Token::Identifier(name) => {
                            vars.push(VarDecl {
                                name: name.clone(),
                                type_def: *type_def,
                                value: None,
                            });
                        }
                        _ => return Err("Couldn't construct a vars sequence".to_string()),
                    };
                    self.advance();
                }
                Ok(Stmt::VarsDecl(vars))
            }
            _ => Ok(Stmt::VarDecl(VarDecl {
                name,
                type_def: *type_def,
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
            left = Expr::BinaryOp(BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            });
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

impl AstNode {
    pub fn eval(&self, environment: &mut Environment) -> Result<(), String> {
        match self {
            AstNode::Program(stmts) => {
                for stmt in stmts {
                    stmt.eval(environment)?;
                }
                Ok(())
            }
            AstNode::Stmt(stmt) => {
                stmt.eval(environment)?;
                Ok(())
            }
            AstNode::Expr(_) => Err("Program couldn't exit with only expr".to_string()),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum EvalResult {
    Normal,
    Break,
}

impl Stmt {
    pub fn eval(&self, environment: &mut Environment) -> Result<EvalResult, String> {
        match self {
            Stmt::VarDecl(var_decl) => {
                if let Some(value) = &var_decl.value {
                    let eval_result = Some(value.eval(environment)?);
                    environment.new_var(&var_decl.name, eval_result, var_decl.type_def);
                } else {
                    environment.new_var(&var_decl.name, None, var_decl.type_def);
                }
            }
            Stmt::VarsDecl(var_decls) => {
                for var_decl in var_decls {
                    Stmt::VarDecl(var_decl.clone()).eval(environment)?;
                }
            }
            Stmt::Assign { name, value } => {
                let value = value.eval(environment)?;
                environment.assign_var(name, value);
            }
            Stmt::Alg { name, body } => todo!(),
            Stmt::Condition {
                condition,
                left,
                right,
            } => {
                execute_condition(
                    left,
                    right,
                    check_condition(condition, environment)?,
                    environment,
                )?;
            }
            Stmt::Loop { condition, body } => {
                if let Some(condition) = condition {
                    while check_condition(condition, environment)? {
                        if EvalResult::Break == execute_body(body, environment)? {
                            break;
                        };
                    }
                } else {
                    loop {
                        if EvalResult::Break == execute_body(body, environment)? {
                            break;
                        };
                    }
                }
            }
            Stmt::ForLoop {
                var,
                start,
                end,
                body,
            } => {
                let step = 1;
                let start = if let Literal::Int(start) = start.eval(environment)? {
                    start
                } else {
                    return Err(format!("{start:?} must be an integer value in loop"));
                };
                let end = if let Literal::Int(end) = end.eval(environment)? {
                    end
                } else {
                    return Err(format!("{end:?} must be an integer value in loop"));
                };
                environment.new_var(var, Some(Literal::Int(start)), TypeDefinition::Int);
                while {
                    if let Some(Literal::Int(i)) = environment.get_value(var) {
                        i != end
                    } else {
                        false
                    }
                } {
                    if EvalResult::Break == execute_body(body, environment)? {
                        break;
                    };
                    if let Some(Literal::Int(i)) = environment.get_value(var) {
                        environment.assign_var(var, Literal::Int(i + step));
                    }
                }
            }
            Stmt::Repeat {
                condition,
                count,
                body,
            } => {
                let mut times = if let Literal::Int(times) = count.eval(environment)? {
                    times
                } else {
                    return Err(format!("{count:?} must be an integer value in loop"));
                };
                loop {
                    if EvalResult::Break == execute_body(body, environment)? {
                        break;
                    };
                    if times == 0 {
                        break;
                    }
                    if let Some(condition) = condition
                        && !check_condition(condition, environment)?
                    {
                        break;
                    }
                    times -= 1;
                }
            }
            Stmt::Break => {
                return Ok(EvalResult::Break);
            }

            Stmt::Output { values } => {
                println!();
                for value in values {
                    if Expr::NewLine == *value {
                        println!();
                    } else {
                        print!("{}", value.eval(environment)?);
                    }
                }
            }
        }
        Ok(EvalResult::Normal)
    }
}

fn check_condition(condition: &Expr, environment: &mut Environment) -> Result<bool, String> {
    let condition_val = condition.eval(environment)?;
    match condition_val {
        Literal::Bool(value) => Ok(value),
        _ => Err(format!("{condition_val:?} must be a boolean value")),
    }
}

fn execute_condition(
    left: &[Stmt],
    right: &Option<Vec<Stmt>>,
    condition: bool,
    environment: &mut Environment,
) -> Result<(), String> {
    if condition {
        execute_body(left, environment)?;
    } else if let Some(right) = right {
        execute_body(right, environment)?;
    }
    Ok(())
}

impl BinaryOp {
    pub fn eval(&self, environment: &mut Environment) -> Result<Literal, String> {
        let left_val = self.left.eval(environment)?;

        let right_val = self.right.eval(environment)?;

        match (&left_val, self.op, &right_val) {
            //Equal operations
            (Literal::Bool(left), Operator::EqualBool, Literal::Bool(right)) => {
                Ok(Literal::Bool(left == right))
            }
            (Literal::Char(left), Operator::EqualBool, Literal::Char(right)) => {
                Ok(Literal::Bool(left == right))
            }
            (Literal::Float(left), Operator::EqualBool, Literal::Float(right)) => {
                Ok(Literal::Bool(left == right))
            }
            (Literal::Int(left), Operator::EqualBool, Literal::Int(right)) => {
                Ok(Literal::Bool(left == right))
            }
            (Literal::String(left), Operator::EqualBool, Literal::String(right)) => {
                Ok(Literal::Bool(left == right))
            }
            //Not equal operations
            (Literal::Bool(left), Operator::NotEqual, Literal::Bool(right)) => {
                Ok(Literal::Bool(left != right))
            }
            (Literal::Char(left), Operator::NotEqual, Literal::Char(right)) => {
                Ok(Literal::Bool(left != right))
            }
            (Literal::Float(left), Operator::NotEqual, Literal::Float(right)) => {
                Ok(Literal::Bool(left != right))
            }
            (Literal::Int(left), Operator::NotEqual, Literal::Int(right)) => {
                Ok(Literal::Bool(left != right))
            }
            (Literal::String(left), Operator::NotEqual, Literal::String(right)) => {
                Ok(Literal::Bool(left != right))
            }
            //Float operations
            (Literal::Float(left), Operator::Plus, Literal::Float(right)) => {
                Ok(Literal::Float(left + right))
            }
            (Literal::Float(left), Operator::Minus, Literal::Float(right)) => {
                Ok(Literal::Float(left - right))
            }
            (Literal::Float(left), Operator::Multiply, Literal::Float(right)) => {
                Ok(Literal::Float(left * right))
            }
            (Literal::Float(left), Operator::Divide, Literal::Float(right)) => {
                Ok(Literal::Float(left / right))
            }
            (Literal::Float(left), Operator::Greater, Literal::Float(right)) => {
                Ok(Literal::Bool(left > right))
            }
            (Literal::Float(left), Operator::GreaterOrEqual, Literal::Float(right)) => {
                Ok(Literal::Bool(left >= right))
            }
            (Literal::Float(left), Operator::Less, Literal::Float(right)) => {
                Ok(Literal::Bool(left < right))
            }
            (Literal::Float(left), Operator::LessOrEqual, Literal::Float(right)) => {
                Ok(Literal::Bool(left <= right))
            }
            //Int operations
            (Literal::Int(left), Operator::Plus, Literal::Int(right)) => {
                Ok(Literal::Int(left + right))
            }
            (Literal::Int(left), Operator::Minus, Literal::Int(right)) => {
                Ok(Literal::Int(left - right))
            }
            (Literal::Int(left), Operator::Multiply, Literal::Int(right)) => {
                Ok(Literal::Int(left * right))
            }
            (Literal::Int(left), Operator::Divide, Literal::Int(right)) => {
                Ok(Literal::Int(left / right))
            }
            (Literal::Int(left), Operator::Greater, Literal::Int(right)) => {
                Ok(Literal::Bool(left > right))
            }
            (Literal::Int(left), Operator::GreaterOrEqual, Literal::Int(right)) => {
                Ok(Literal::Bool(left >= right))
            }
            (Literal::Int(left), Operator::Less, Literal::Int(right)) => {
                Ok(Literal::Bool(left < right))
            }
            (Literal::Int(left), Operator::LessOrEqual, Literal::Int(right)) => {
                Ok(Literal::Bool(left <= right))
            }

            _ => Err(format!(
                "Invalid operation: {:?} {:?} {:?}",
                left_val, self.op, right_val
            )),
        }
    }
}

impl Expr {
    pub fn eval(&self, environment: &mut Environment) -> Result<Literal, String> {
        match self {
            Expr::Literal(literal) => Ok(literal.clone()),
            Expr::Identifier(name) => {
                if let Some(value) = environment.get_value(name) {
                    Ok(value.clone())
                } else {
                    Err(format!("Undefined variable: {name}"))
                }
            }
            Expr::BinaryOp(binary_op) => binary_op.eval(environment),
            Expr::NewLine => return Err(format!("New line couldn't be Literal")),
            Expr::FunctionCall(call) => {
                let mut params_literals = vec![];
                for param in call.args.iter() {
                    params_literals.push(param.eval(environment)?);
                }
                let function = environment
                    .get_function(&call.name)
                    .ok_or(format!("Function with name {} not found", &call.name))?
                    .clone();
                check_params(
                    &function
                        .params
                        .iter()
                        .map(|param| param.1)
                        .collect::<Vec<TypeDefinition>>(),
                    &params_literals
                        .iter()
                        .map(|param| param.get_type())
                        .collect::<Vec<TypeDefinition>>(),
                )?;
                for (value, name) in params_literals
                    .iter()
                    .zip(function.params.iter().map(|param| param.0.clone()))
                {
                    environment.assign_var(&name, value.clone());
                }
                execute_body(&function.body, environment)?;
                for name in function.params.iter().map(|param| param.0.clone()) {
                    environment.remove_var(&name)?;
                }

                todo!()
            }
        }
    }
}

fn check_params(params: &[TypeDefinition], params1: &[TypeDefinition]) -> Result<(), String> {
    if params.len() != params1.len() {
        return Err(format!("Len of params of function call is different"));
    }

    for (param, param1) in params.iter().zip(params1.iter()) {
        if param != param1 {
            return Err(format!("type of {} != {}", param, param1));
        }
    }

    Ok(())
}

#[derive(Debug, PartialEq, Clone)]
pub struct Function {
    params: Vec<(String, TypeDefinition)>,
    body: Vec<Stmt>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Variable {
    pub type_def: TypeDefinition,
    pub value: Option<Literal>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Environment {
    variables: HashMap<String, Variable>,
    functions: HashMap<String, Function>,
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    fn new_var(&mut self, name: &str, value: Option<Literal>, type_def: TypeDefinition) {
        self.variables
            .insert(name.to_string(), Variable { type_def, value });
    }

    fn assign_var(&mut self, name: &str, value: Literal) {
        if self.get_var_type(name) == Some(value.get_type()) {
            self.variables.insert(
                name.to_string(),
                Variable {
                    type_def: value.get_type(),
                    value: Some(value),
                },
            );
        }
    }

    fn get_var_type(&self, name: &str) -> Option<TypeDefinition> {
        if let Some(var) = self.get_var(name) {
            return Some(var.type_def);
        }
        None
    }

    fn check_var_exist(&mut self, name: &str) -> bool {
        self.variables.contains_key(name)
    }

    fn get_var(&self, name: &str) -> Option<Variable> {
        Some(self.variables.get(name)?.clone())
    }
    fn get_value(&self, name: &str) -> Option<Literal> {
        self.variables.get(name)?.clone().value
    }

    fn remove_var(&mut self, name: &str) -> Result<(), String> {
        self.variables
            .remove(name)
            .map(|_| ())
            .ok_or_else(|| format!("No such variable exists"))
    }

    fn register_function(&mut self, name: &str, function: Function) {
        self.functions.insert(name.to_string(), function);
    }

    fn get_function(&self, name: &str) -> Option<&Function> {
        self.functions.get(name)
    }
}

fn execute_body(body: &[Stmt], environment: &mut Environment) -> Result<EvalResult, String> {
    for stmt in body {
        if EvalResult::Break == stmt.eval(environment)? {
            return Ok(EvalResult::Break);
        }
    }
    Ok(EvalResult::Normal)
}

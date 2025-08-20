use std::{
    cell::RefCell,
    env,
    fmt::{Display, format},
    rc::Rc,
};

use hashbrown::HashMap;
use indexmap::IndexMap;
use log::info;

use crate::{
    interpreter::Interpreter,
    lexer::{
        self, Condition, Delimiter, FunctionParamType, IO, Keyword, Loop, Operator, Range, Token,
        TypeDefinition,
    },
};

#[derive(Debug, PartialEq, Clone)]
pub enum AstNode {
    Program(Vec<Stmt>),
    Stmt(Stmt),
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
    Alg(Function),
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
        body: Box<AstNode>,
    },
    Repeat {
        condition: Option<Expr>,
        count: Expr,
        body: Box<AstNode>,
    },
    Output {
        values: Vec<Expr>,
    },
    FunctionCall(FunctionCall),
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

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionParameter {
    pub type_definition: TypeDefinition,
    pub result_type: FunctionParamType,
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

    pub fn parse(&mut self) -> Result<AstNode, (Vec<Stmt>, String)> {
        let mut statements = Vec::new();
        while !self.is_eof() {
            statements.push(match self.parse_stmt() {
                Ok(stmt) => stmt,
                Err(err) => {
                    return Err((statements, err));
                }
            })
        }
        Ok(AstNode::Program(statements))
    }

    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        match self.current_token().clone() {
            Token::Keyword(Keyword::Function(lexer::Function::Alg)) => self.parse_alg(),
            Token::Keyword(Keyword::Condition(Condition::If)) => self.parse_condition(),
            Token::Keyword(Keyword::Loop(Loop::Start)) => self.parse_loop(),
            Token::Keyword(Keyword::Loop(Loop::Break)) => Ok(Stmt::Break),
            Token::Keyword(Keyword::TypeDef(type_def)) => self.parse_var_decl(&type_def),
            Token::Identifier(name) => {
                if self.next_token().is_operator(Operator::Assignment) {
                    self.parse_assign()
                } else {
                    self.parse_function_call_stmt(&name)
                }
            }
            Token::Keyword(Keyword::IO(io_keyword)) => self.parse_io(io_keyword),
            _ => Err(format!("Unexpected token: {:?}", self.current_token())),
        }
    }

    fn parse_function_call_stmt(&mut self, name: &str) -> Result<Stmt, String> {
        Ok(Stmt::FunctionCall(self.parse_function_call(name)?))
    }

    fn parse_function_call_expr(&mut self, name: &str) -> Result<Expr, String> {
        Ok(Expr::FunctionCall(self.parse_function_call(name)?))
    }

    fn parse_function_call(&mut self, name: &str) -> Result<FunctionCall, String> {
        //Skip name
        self.advance();

        //Parse arguments
        let mut args: Vec<Expr> = vec![];
        if self
            .current_token()
            .is_delimiter(Delimiter::ParenthesisOpen)
        {
            //Skip "("
            self.advance();

            while !self
                .current_token()
                .is_delimiter(Delimiter::ParenthesisClose)
            {
                args.push(self.parse_expr()?);
                if self.current_token().is_delimiter(Delimiter::Comma) {
                    self.advance()
                } else {
                    break;
                }
            }

            //Skip ")"
            self.advance();
        }
        let name = name.to_string();
        Ok(FunctionCall { name, args })
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
            body: Box::new(AstNode::Program(statements)),
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
            body: Box::new(AstNode::Program(statements)),
        })
    }

    fn parse_alg(&mut self) -> Result<Stmt, String> {
        self.advance(); // Skip Alg
        let return_type =
            if let Token::Keyword(Keyword::TypeDef(return_type)) = self.current_token() {
                let return_type = *return_type;
                self.advance();
                Some(return_type)
            } else {
                None
            };
        info!("Function return type: {:?}", return_type);
        let name = match self.current_token().clone() {
            Token::Identifier(name) => {
                self.advance();
                name.clone()
            }
            _ => "main".to_string(),
        };
        info!("Function name: {name}");
        let mut params: IndexMap<String, FunctionParameter> = IndexMap::new();
        if Token::Delimiter(Delimiter::ParenthesisOpen) == *self.current_token() {
            self.advance();
            info!(
                "starting parsing function params at token {}",
                self.position
            );
            while !self.check(&Token::Delimiter(Delimiter::ParenthesisClose)) && !self.is_eof() {
                info!("starting parsing function param at token {}", self.position);
                let result_type: FunctionParamType = {
                    if let Token::Keyword(Keyword::Function(lexer::Function::FunctionParamType(
                        result_type,
                    ))) = *self.current_token()
                    {
                        //Skip param result type
                        self.advance();
                        if result_type == FunctionParamType::ArgumentParam {
                            if Token::Keyword(Keyword::Function(
                                lexer::Function::FunctionParamType(FunctionParamType::ResultParam),
                            )) == *self.current_token()
                            {
                                FunctionParamType::ArgumentResultParam
                            } else {
                                result_type
                            }
                        } else {
                            result_type
                        }
                    } else {
                        FunctionParamType::ArgumentParam
                    }
                };

                info!("Function result param type parsed: {:?}", result_type);

                let type_definition: TypeDefinition = {
                    if let Token::Keyword(Keyword::TypeDef(type_def)) = self.current_token() {
                        *type_def
                    } else {
                        return Err(format!("Typedef for function parameter required"));
                    }
                };

                info!("Function param typedef parsed: {:?}", type_definition);

                //Skip typedef
                self.advance();

                let names: Vec<String> = {
                    let mut names = vec![];
                    while let Token::Identifier(name) = self.current_token() {
                        info!("Pushing name {}, current token {}", &name, self.position);
                        names.push(name.clone());
                        //Skip name
                        self.advance();
                        if *self.current_token() != Token::Delimiter(Delimiter::Comma) {
                            break;
                        }
                        //skip comma
                        self.advance();
                    }

                    names
                };

                params.extend(names.iter().map(|name| {
                    (
                        name.clone(),
                        FunctionParameter {
                            type_definition,
                            result_type,
                        },
                    )
                }));
            }
        }
        info!(
            "Function params parsed: {:#?}, token stoped: {}",
            params, self.position
        );

        self.expect(Token::Delimiter(Delimiter::ParenthesisClose))?;

        if self.check(&Token::Keyword(Keyword::Function(lexer::Function::Expects))) {
            //skip Expects token
            self.advance();
            todo!("Parse alg expects token")
        }

        self.expect(Token::Keyword(Keyword::Function(lexer::Function::Start)))?;

        let mut body = Vec::new();
        while !self.check(&Token::Keyword(Keyword::Function(lexer::Function::Stop)))
            && !self.is_eof()
        {
            body.push(self.parse_stmt()?);
        }
        let body = Box::new(AstNode::Program(body));
        self.expect(Token::Keyword(Keyword::Function(lexer::Function::Stop)))?;
        Ok(Stmt::Alg(Function {
            name,
            body,
            return_type,
            params,
        }))
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
                if self.next_token().is_delimiter(Delimiter::ParenthesisOpen) {
                    return self.parse_function_call_expr(&name);
                }
                self.advance();
                Ok(Expr::Identifier(name.clone()))
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

    fn next_token(&self) -> &Token {
        self.tokens.get(self.position + 1).unwrap_or(&Token::Eof)
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
    pub fn eval(&self, environment: &mut Environment) -> Result<EvalResult, String> {
        match self {
            AstNode::Program(body) => {
                for stmt in body {
                    let eval_result = stmt.eval(environment)?;
                    match eval_result {
                        EvalResult::Procedure => {}
                        EvalResult::Literal(literal) => return Ok(EvalResult::Literal(literal)),
                        EvalResult::Break => return Ok(EvalResult::Break),
                    }
                }
                Ok(EvalResult::Procedure)
            }
            AstNode::Stmt(stmt) => stmt.eval(environment),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum EvalResult {
    Procedure,
    Literal(Literal),
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
            Stmt::Alg(_) => {}
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
                    if EvalResult::Break == body.eval(environment)? {
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
                    if EvalResult::Break == body.eval(environment)? {
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
            Stmt::FunctionCall(call) => {
                function_call(call, environment)?;
            }
        }
        Ok(EvalResult::Procedure)
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
            Expr::FunctionCall(call) => match function_call(call, environment)? {
                FunctionResult::Literal(literal) => Ok(literal),
                FunctionResult::Procedure => Err(format!("This alg is procedure not function")),
            },
        }
    }
}

fn function_call(
    call: &FunctionCall,
    environment: &mut Environment,
) -> Result<FunctionResult, String> {
    let function: FunctionVariant = environment
        .get_function(&call.name)
        .ok_or(format!(
            "Couldn't call undefined function with name {}",
            &call.name
        ))?
        .clone();

    let run_function =
        |args_expr: &Vec<Expr>,
         params: &IndexMap<String, FunctionParameter>,
         return_type: Option<TypeDefinition>,
         environment: &mut Environment,
         function: Box<dyn FnOnce(&mut Environment) -> Result<Option<Literal>, String>>|
         -> Result<FunctionResult, String> {
            let mut args: HashMap<usize, Literal> = HashMap::new();
            for (i, (expr, param)) in args_expr.iter().zip(params.values()).enumerate() {
                if param.result_type != FunctionParamType::ResultParam {
                    let value = expr.eval(environment)?;
                    args.insert(i, value);
                }
            }

            //check param count
            if args_expr.len() != params.len() {
                return Err(format!(
                    "params and args mismatch args: {:?}, params: {:?}",
                    args_expr, params
                ));
            }

            //check param types
            for (i, arg) in args.iter() {
                if params
                    .get_index(*i)
                    .ok_or(format!("Couldn't "))?
                    .1
                    .type_definition
                    != arg.get_type()
                {
                    return Err(format!(
                        "params and args mismatch args: {:?}, params: {:?}",
                        args, params
                    ));
                }
            }

            //Creating function scope...
            let mut scope = Environment::new();
            scope.functions = environment.functions.clone();

            let mut value_to_return_from_function: Vec<String> = vec![];

            //Map return types
            for (i, (name, parameter)) in params.iter().enumerate() {
                match parameter.result_type {
                    FunctionParamType::ResultParam => {
                        if environment.get_var_type(name) != Some(parameter.type_definition) {
                            return Err(format!(
                                "params and args mismatch args: {:?}, params: {:?}",
                                args, params
                            ));
                        }
                        scope.new_var(name, None, parameter.type_definition);
                        value_to_return_from_function.push(name.clone());
                    }
                    FunctionParamType::ArgumentParam => {
                        scope.new_var(
                            name,
                            Some(args.get(&i).unwrap().clone()),
                            parameter.type_definition,
                        );
                    }
                    FunctionParamType::ArgumentResultParam => {
                        scope.new_var(
                            name,
                            Some(args.get(&i).unwrap().clone()),
                            parameter.type_definition,
                        );
                        value_to_return_from_function.push(name.clone());
                    }
                }
            }

            let is_not_procedure = if let Some(return_type) = return_type {
                scope.new_var("знач".into(), None, return_type);
                true
            } else {
                false
            };

            //Execute function
            let value = function(&mut scope)?;

            //Return values
            for name in value_to_return_from_function {
                let value = scope
                    .get_var(&name)
                    .ok_or(format!("Couldn't find variable"))?
                    .value
                    .ok_or(format!("Variable value is None"))?;
                environment.assign_var(&name, value);
            }

            if is_not_procedure {
                if let Some(value) = value {
                    return Ok(FunctionResult::Literal(value));
                }
                let value = scope
                    .get_value("знач")
                    .ok_or(format!("Value of alg func is nothing"))?;
                return Ok(FunctionResult::Literal(value));
            } else {
                return Ok(FunctionResult::Procedure);
            }
        };

    match function {
        FunctionVariant::Native(NativeFunction {
            native_function,
            params,
            return_type,
        }) => run_function(
            &call.args,
            &params,
            return_type,
            environment,
            Box::new(move |env: &mut Environment| native_function.borrow_mut()(env)),
        ),
        FunctionVariant::Kumir(function) => run_function(
            &call.args,
            &function.params,
            function.return_type,
            environment,
            Box::new(
                |environment: &mut Environment| match function.body.eval(environment)? {
                    EvalResult::Literal(literal) => Ok(Some(literal)),
                    EvalResult::Procedure => Ok(None),
                    EvalResult::Break => Ok(None),
                },
            ),
        ),
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Function {
    pub name: String,
    pub body: Box<AstNode>,
    pub params: IndexMap<String, FunctionParameter>,
    pub return_type: Option<TypeDefinition>,
}

pub type ClonableFnMut =
    Rc<RefCell<dyn FnMut(&mut Environment) -> Result<Option<Literal>, String>>>;

#[derive(Clone)]
pub struct NativeFunction {
    pub params: IndexMap<String, FunctionParameter>,
    pub return_type: Option<TypeDefinition>,
    pub native_function: ClonableFnMut,
}

impl std::fmt::Debug for NativeFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeFunction")
            .field("params", &self.params)
            .field("return_type", &self.return_type)
            .finish()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum FunctionResult {
    Literal(Literal),
    Procedure,
}

#[derive(Debug, Clone)]
pub enum FunctionVariant {
    Native(NativeFunction),
    Kumir(Function),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Variable {
    pub type_def: TypeDefinition,
    pub value: Option<Literal>,
}

#[derive(Debug, Clone)]
pub struct Environment {
    variables: HashMap<String, Variable>,
    functions: HashMap<String, FunctionVariant>,
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

    pub fn new_var(&mut self, name: &str, value: Option<Literal>, type_def: TypeDefinition) {
        self.variables
            .insert(name.to_string(), Variable { type_def, value });
    }

    pub fn assign_var(&mut self, name: &str, value: Literal) {
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

    pub fn get_var_type(&self, name: &str) -> Option<TypeDefinition> {
        if let Some(var) = self.get_var(name) {
            return Some(var.type_def);
        }
        None
    }

    pub fn check_var_exist(&mut self, name: &str) -> bool {
        self.variables.contains_key(name)
    }

    pub fn get_var(&self, name: &str) -> Option<Variable> {
        Some(self.variables.get(name)?.clone())
    }
    pub fn get_value(&self, name: &str) -> Option<Literal> {
        self.variables.get(name)?.clone().value
    }

    pub fn remove_var(&mut self, name: &str) -> Result<(), String> {
        self.variables
            .remove(name)
            .map(|_| ())
            .ok_or_else(|| format!("No such variable exists"))
    }

    pub fn register_function(&mut self, name: &str, function: FunctionVariant) {
        self.functions.insert(name.to_string(), function);
    }

    pub fn get_function(&self, name: &str) -> Option<&FunctionVariant> {
        self.functions.get(name)
    }
}

fn execute_body(body: &[Stmt], environment: &mut Environment) -> Result<EvalResult, String> {
    for stmt in body {
        let eval_result = stmt.eval(environment)?;
        match eval_result {
            EvalResult::Procedure => {}
            EvalResult::Literal(literal) => return Ok(EvalResult::Literal(literal)),
            EvalResult::Break => return Ok(EvalResult::Break),
        }
    }
    Ok(EvalResult::Procedure)
}

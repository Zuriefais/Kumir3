use crate::ast::*;

use indexmap::IndexMap;
use log::info;

use crate::lexer::{
    self, Delimiter, FunctionParamType, IO, Keyword, Operator, Range, Token, TypeDefinition,
};

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
            Token::Keyword(Keyword::Function(lexer::Function::ImportNamespace)) => {
                self.parse_import_namespace()
            }
            Token::Keyword(Keyword::Condition(lexer::Condition::If)) => self.parse_condition(),
            Token::Keyword(Keyword::Loop(lexer::Loop::Start)) => self.parse_loop(),
            Token::Keyword(Keyword::Loop(lexer::Loop::Break)) => Ok(Stmt::Break),
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

    fn parse_import_namespace(&mut self) -> Result<Stmt, String> {
        //Skip import namespace token
        self.advance();
        let name = self.current_token().identifier().ok_or(format!(
            "Error parsing import namespace expected identifier found {:?}",
            self.current_token()
        ))?;
        self.advance();
        Ok(Stmt::ImportNamespace(ImportNamespace { name }))
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
            IO::ChangeLine => Err("Change line couldn't be Statement".to_string()),
        }
    }

    fn parse_input(&mut self) -> Result<Stmt, String> {
        self.advance();

        todo!()
    }

    fn parse_output(&mut self) -> Result<Stmt, String> {
        //Skip Output keyword
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
                    if !self.current_token().is_delimiter(Delimiter::Comma) {
                        break;
                    }
                }
            }
        }

        Ok(Stmt::Output { values })
    }

    fn parse_condition(&mut self) -> Result<Stmt, String> {
        self.advance();
        let condition = self.parse_expr()?;
        self.expect(Token::Keyword(Keyword::Condition(lexer::Condition::Then)))?;

        let mut left = Vec::new();
        while !self.check(&Token::Keyword(Keyword::Condition(lexer::Condition::Else)))
            && !self.check(&Token::Keyword(Keyword::Condition(
                lexer::Condition::EndCondition,
            )))
        {
            left.push(self.parse_stmt()?);
        }

        let right = if self.check(&Token::Keyword(Keyword::Condition(lexer::Condition::Else))) {
            self.advance();
            let mut else_branch = Vec::new();
            while !self.check(&Token::Keyword(Keyword::Condition(
                lexer::Condition::EndCondition,
            ))) {
                else_branch.push(self.parse_stmt()?);
            }
            Some(Box::new(AstNode::Program(else_branch)))
        } else {
            None
        };

        self.expect(Token::Keyword(Keyword::Condition(
            lexer::Condition::EndCondition,
        )))?;
        let left = Box::new(AstNode::Program(left));
        Ok(Stmt::Condition(Condition {
            condition,
            left,
            right,
        }))
    }

    fn parse_loop(&mut self) -> Result<Stmt, String> {
        self.advance();
        match self.current_token() {
            Token::Keyword(Keyword::Loop(lexer::Loop::While)) => self.parse_while_loop(),
            Token::Int(times) => self.parse_repeat_loop(Expr::Literal(Literal::Int(*times))),
            Token::Keyword(Keyword::Range(Range::For)) => self.parse_for_loop(),

            _ => self.parse_simple_loop(),
        }
    }

    fn parse_simple_loop(&mut self) -> Result<Stmt, String> {
        self.advance();
        let mut statements = Vec::new();
        let mut condition = None;
        while *self.current_token() != Token::Keyword(Keyword::Loop(lexer::Loop::End)) {
            if *self.current_token() != Token::Keyword(Keyword::Loop(lexer::Loop::EndIf)) {
                self.advance();
                condition = Some(self.parse_expr()?);
            } else {
                statements.push(self.parse_stmt()?);
            }
        }
        let body = Box::new(AstNode::Program(statements));
        Ok(Stmt::Loop(Loop { condition, body }))
    }

    fn parse_while_loop(&mut self) -> Result<Stmt, String> {
        //Skip Loop start token
        self.advance();
        let condition = Some(self.parse_expr()?);
        let mut statements = Vec::new();
        while *self.current_token() != Token::Keyword(Keyword::Loop(lexer::Loop::End)) {
            statements.push(self.parse_stmt()?);
        }
        //Skip loop end token
        self.advance();
        let body = Box::new(AstNode::Program(statements));
        Ok(Stmt::Loop(Loop { condition, body }))
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
        while *self.current_token() != Token::Keyword(Keyword::Loop(lexer::Loop::End)) {
            statements.push(self.parse_stmt()?);
        }
        self.advance();
        Ok(Stmt::ForLoop(ForLoop {
            var,
            start,
            end,
            body: Box::new(AstNode::Program(statements)),
        }))
    }

    fn parse_repeat_loop(&mut self, count: Expr) -> Result<Stmt, String> {
        self.advance(); // Skip the count
        self.expect(Token::Keyword(Keyword::Loop(lexer::Loop::Times)))?;
        let mut statements = Vec::new();
        let mut condition = None;
        while *self.current_token() != Token::Keyword(Keyword::Loop(lexer::Loop::End)) {
            if *self.current_token() != Token::Keyword(Keyword::Loop(lexer::Loop::EndIf)) {
                self.advance();
                condition = Some(self.parse_expr()?);
            } else {
                statements.push(self.parse_stmt()?);
            }
        }
        self.advance(); // Skip Loop::End
        Ok(Stmt::RepeatLoop(RepeatLoop {
            condition,
            count,
            body: Box::new(AstNode::Program(statements)),
        }))
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
                        return Err("Typedef for function parameter required".to_string());
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

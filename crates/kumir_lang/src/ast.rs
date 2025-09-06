use std::{
    cell::RefCell,
    fmt::{Display, format},
    rc::Rc,
    sync::{Arc, atomic::AtomicBool},
};

use hashbrown::HashMap;
use indexmap::IndexMap;
use log::info;

use crate::lexer::{FunctionParamType, Operator, TypeDefinition};

#[derive(Debug, PartialEq, Clone)]
pub enum AstNode {
    Program(Vec<Stmt>),
    Stmt(Stmt),
}

impl AstNode {
    pub fn eval(&self, environment: &Rc<RefCell<Environment>>) -> Result<EvalResult, String> {
        let scope: Rc<RefCell<Environment>> = Rc::new(RefCell::new(Environment {
            environment: Some(environment.clone()),
            variables: Default::default(),
            functions: environment.borrow().functions.clone(),
            namespaces: environment.borrow().namespaces.clone(),
            kill_flag: environment.borrow().kill_flag.clone(),
        }));
        info!("scope variables: {:#?}", scope.borrow().get_all_vars());
        match self {
            AstNode::Program(body) => {
                for stmt in body {
                    let eval_result = stmt.eval(&scope)?;
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
pub struct VarDecl {
    pub name: String,
    pub type_def: TypeDefinition,
    pub value: Option<Expr>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
    VarDecl(VarDecl),
    VarsDecl(Vec<VarDecl>),
    Assign { name: String, value: Expr },
    Alg(Function),
    Condition(Condition),
    Loop(Loop),
    ForLoop(ForLoop),
    RepeatLoop(RepeatLoop),
    Output { values: Vec<Expr> },
    FunctionCall(FunctionCall),
    ImportNamespace(ImportNamespace),
    Break,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ImportNamespace {
    pub name: String,
}

impl ImportNamespace {
    fn eval(&self, environment: &Rc<RefCell<Environment>>) -> Result<(), String> {
        environment.borrow_mut().import_namespace(&self.name)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Loop {
    pub condition: Option<Expr>,
    pub body: Box<AstNode>,
}

fn check_condition(
    condition: &Expr,
    environment: &Rc<RefCell<Environment>>,
) -> Result<bool, String> {
    let condition_val = condition.eval(environment)?;
    match condition_val {
        Literal::Bool(value) => Ok(value),
        _ => Err(format!("{condition_val:?} must be a boolean value")),
    }
}

impl Loop {
    fn eval(&self, environment: &Rc<RefCell<Environment>>) -> Result<(), String> {
        if let Some(condition) = self.condition.as_ref() {
            while check_condition(&condition, environment)? {
                if EvalResult::Break == self.body.eval(environment)? {
                    break;
                };
            }
        } else {
            loop {
                if EvalResult::Break == self.body.eval(environment)? {
                    break;
                };
            }
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ForLoop {
    pub var: String,
    pub start: Expr,
    pub end: Expr,
    pub body: Box<AstNode>,
}

impl ForLoop {
    fn eval(&self, environment: &Rc<RefCell<Environment>>) -> Result<(), String> {
        let step = 1;
        let start = if let Literal::Int(start) = self.start.eval(environment)? {
            start
        } else {
            return Err(format!("{:?} must be an integer value in loop", self.start));
        };
        let end = if let Literal::Int(end) = self.end.eval(environment)? {
            end
        } else {
            return Err(format!("{:?} must be an integer value in loop", self.end));
        };
        let mut environment_mut = environment.borrow_mut();
        environment_mut.new_var(&self.var, Some(Literal::Int(start)), TypeDefinition::Int);
        while {
            if let Some(Literal::Int(i)) = environment_mut.get_value(&self.var) {
                i != end
            } else {
                false
            }
        } {
            if EvalResult::Break == self.body.eval(environment)? {
                break;
            };
            if let Some(Literal::Int(i)) = environment_mut.get_value(&self.var) {
                environment_mut.assign_var(&self.var, Literal::Int(i + step))?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Condition {
    pub condition: Expr,
    pub left: Box<AstNode>,
    pub right: Option<Box<AstNode>>,
}

impl Condition {
    fn eval(&self, environment: &Rc<RefCell<Environment>>) -> Result<(), String> {
        if {
            let condition_val = self.condition.eval(environment)?;
            match condition_val {
                Literal::Bool(value) => value,
                _ => return Err(format!("{condition_val:?} must be a boolean value")),
            }
        } {
            self.left.eval(environment)?;
        } else if let Some(right) = self.right.as_ref() {
            right.eval(environment)?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct RepeatLoop {
    pub condition: Option<Expr>,
    pub count: Expr,
    pub body: Box<AstNode>,
}

impl RepeatLoop {
    fn eval(&self, environment: &Rc<RefCell<Environment>>) -> Result<(), String> {
        let mut times = if let Literal::Int(times) = self.count.eval(environment)? {
            times
        } else {
            return Err(format!("{:?} must be an integer value in loop", self.count));
        };
        loop {
            if EvalResult::Break == self.body.eval(environment)? {
                break;
            };
            if times == 0 {
                break;
            }
            if let Some(condition) = self.condition.as_ref() {
                if !check_condition(condition, environment)? {
                    break;
                }
            }
            times -= 1;
        }
        Ok(())
    }
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

impl FunctionCall {
    fn eval(&self, environment: &Rc<RefCell<Environment>>) -> Result<FunctionResult, String> {
        let call = self;
        let function: FunctionVariant = environment
            .borrow()
            .get_function(&call.name)
            .ok_or(format!(
                "Couldn't call undefined function with name {}",
                &call.name
            ))?
            .clone();

        let run_function = |args_expr: &Vec<Expr>,
                            params: &IndexMap<String, FunctionParameter>,
                            return_type: Option<TypeDefinition>,
                            environment: &Rc<RefCell<Environment>>,
                            function: Box<
            dyn FnOnce(&Rc<RefCell<Environment>>) -> Result<Option<Literal>, String>,
        >|
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
            let mut environment = environment.borrow_mut();
            let scope: Rc<RefCell<Environment>> = Rc::new(RefCell::new({
                let mut scope = Environment::default();
                scope.functions = environment.functions.clone();
                scope.namespaces = environment.namespaces.clone();
                scope.kill_flag = environment.kill_flag.clone();
                scope
            }));

            let mut value_to_return_from_function: Vec<String> = vec![];

            //Map return types
            let mut is_function = false;
            {
                let mut scope_mut = scope.borrow_mut();
                for (i, (name, parameter)) in params.iter().enumerate() {
                    match parameter.result_type {
                        FunctionParamType::ResultParam => {
                            if environment.get_var_type(name) != Some(parameter.type_definition) {
                                return Err(format!(
                                    "params and args mismatch args: {:?}, params: {:?}",
                                    args, params
                                ));
                            }
                            scope_mut.new_var(name, None, parameter.type_definition);
                            value_to_return_from_function.push(name.clone());
                        }
                        FunctionParamType::ArgumentParam => {
                            scope_mut.new_var(
                                name,
                                Some(args.get(&i).unwrap().clone()),
                                parameter.type_definition,
                            );
                        }
                        FunctionParamType::ArgumentResultParam => {
                            scope_mut.new_var(
                                name,
                                Some(args.get(&i).unwrap().clone()),
                                parameter.type_definition,
                            );
                            value_to_return_from_function.push(name.clone());
                        }
                    }
                }

                if let Some(return_type) = return_type {
                    scope_mut.new_var("знач".into(), None, return_type);
                    is_function = true;
                }
            }

            //Execute function
            let value = function(&scope)?;
            info!(
                "scope variables after function execution: {:#?}",
                scope.borrow().get_all_vars()
            );
            //Return values
            let scope = scope.borrow_mut();
            for name in value_to_return_from_function {
                let value = scope
                    .get_var(&name)
                    .ok_or(format!("Couldn't find variable"))?
                    .value
                    .ok_or(format!("Variable value is None"))?;
                environment.assign_var(&name, value)?;
            }

            if is_function {
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
                Box::new(move |env: &Rc<RefCell<Environment>>| native_function.borrow_mut()(env)),
            ),
            FunctionVariant::Kumir(function) => run_function(
                &call.args,
                &function.params,
                function.return_type,
                environment,
                Box::new(|environment: &Rc<RefCell<Environment>>| {
                    match function.body.eval(environment)? {
                        EvalResult::Literal(literal) => Ok(Some(literal)),
                        EvalResult::Procedure => Ok(None),
                        EvalResult::Break => Ok(None),
                    }
                }),
            ),
        }
    }
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

#[derive(Debug, PartialEq, Clone)]
pub enum EvalResult {
    Procedure,
    Literal(Literal),
    Break,
}

impl Stmt {
    pub fn eval(&self, environment: &Rc<RefCell<Environment>>) -> Result<EvalResult, String> {
        let kill_flag = environment.borrow().kill_flag.clone();
        if kill_flag.load(std::sync::atomic::Ordering::Relaxed) {
            kill_flag.store(false, std::sync::atomic::Ordering::Relaxed);
            return Err(format!("User interrupt"));
        }
        match self {
            Stmt::VarDecl(var_decl) => {
                let mut environment_mut = environment.borrow_mut();
                if let Some(value) = &var_decl.value {
                    let eval_result = Some(value.eval(environment)?);
                    environment_mut.new_var(&var_decl.name, eval_result, var_decl.type_def);
                } else {
                    environment_mut.new_var(&var_decl.name, None, var_decl.type_def);
                }
            }
            Stmt::VarsDecl(var_decls) => {
                for var_decl in var_decls {
                    Stmt::VarDecl(var_decl.clone()).eval(environment)?;
                }
            }
            Stmt::Assign { name, value } => {
                let value = value.eval(environment)?;
                environment.borrow_mut().assign_var(name, value)?;
            }
            Stmt::Alg(_) => {}
            Stmt::Condition(condition) => {
                condition.eval(environment)?;
            }
            Stmt::Loop(loop_stmt) => loop_stmt.eval(environment)?,
            Stmt::ForLoop(for_loop) => for_loop.eval(environment)?,
            Stmt::RepeatLoop(repeat_loop) => repeat_loop.eval(environment)?,
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
                call.eval(environment)?;
            }
            Stmt::ImportNamespace(import_namespace) => import_namespace.eval(environment)?,
        }
        Ok(EvalResult::Procedure)
    }
}

impl BinaryOp {
    pub fn eval(&self, environment: &Rc<RefCell<Environment>>) -> Result<Literal, String> {
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
    pub fn eval(&self, environment: &Rc<RefCell<Environment>>) -> Result<Literal, String> {
        match self {
            Expr::Literal(literal) => Ok(literal.clone()),
            Expr::Identifier(name) => {
                if environment.borrow().get_function(name).is_some() {
                    let call = FunctionCall {
                        name: name.to_string(),
                        args: vec![],
                    };
                    match call.eval(environment)? {
                        FunctionResult::Literal(literal) => Ok(literal),
                        FunctionResult::Procedure => {
                            Err(format!("This alg is procedure not function"))
                        }
                    }
                } else if let Some(value) = environment.borrow().get_value(name) {
                    Ok(value.clone())
                } else {
                    Err(format!("Undefined variable: {name}"))
                }
            }
            Expr::BinaryOp(binary_op) => binary_op.eval(environment),
            Expr::NewLine => return Err(format!("New line couldn't be Literal")),
            Expr::FunctionCall(call) => match call.eval(environment)? {
                FunctionResult::Literal(literal) => Ok(literal),
                FunctionResult::Procedure => Err(format!("This alg is procedure not function")),
            },
        }
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
    Rc<RefCell<dyn FnMut(&Rc<RefCell<Environment>>) -> Result<Option<Literal>, String>>>;

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

#[derive(Debug, Clone, Default)]
pub struct Namespace {
    functions: HashMap<String, FunctionVariant>,
}

impl Namespace {
    pub fn get_function(&self, name: &str) -> Option<FunctionVariant> {
        self.functions
            .get(&name.to_string())
            .map_or(None, |function| Some(function.clone()))
    }

    pub fn register_function(&mut self, name: &str, function: FunctionVariant) {
        self.functions.insert(name.to_string(), function);
    }

    pub fn register_native_function(&mut self, name: &str, function: NativeFunction) {
        self.register_function(name, FunctionVariant::Native(function));
    }

    pub fn functions(&self) -> &HashMap<String, FunctionVariant> {
        &self.functions
    }
}

#[derive(Debug, Clone)]
pub struct Environment {
    pub environment: Option<Rc<RefCell<Environment>>>,
    pub variables: HashMap<String, Variable>,
    pub namespaces: HashMap<String, Namespace>,
    pub functions: HashMap<String, FunctionVariant>,
    pub kill_flag: Arc<AtomicBool>,
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            environment: None,
            variables: HashMap::new(),
            namespaces: HashMap::new(),
            functions: HashMap::new(),
            kill_flag: Default::default(),
        }
    }
}

impl Environment {
    pub fn new_var(&mut self, name: &str, value: Option<Literal>, type_def: TypeDefinition) {
        self.variables
            .insert(name.to_string(), Variable { type_def, value });
    }

    pub fn assign_var(&mut self, name: &str, value: Literal) -> Result<(), String> {
        if let Some(variable) = self.variables.get_mut(name) {
            if variable.type_def == value.get_type() {
                variable.value = Some(value);
                return Ok(());
            } else {
                return Err(format!(
                    "Type mismatch on assignment, type expected: {}, type received: {}",
                    variable.type_def,
                    value.get_type()
                ));
            }
        }

        if let Some(parent) = self.environment.as_ref() {
            return parent.borrow_mut().assign_var(name, value);
        }

        Err(format!("Cannot assign to undefined variable '{}'", name))
    }

    pub fn get_var_type(&self, name: &str) -> Option<TypeDefinition> {
        if let Some(var) = self.get_var(name) {
            return Some(var.type_def);
        }
        None
    }

    pub fn var_is_some(&mut self, name: &str) -> bool {
        self.get_var(name).is_some()
    }

    pub fn get_var(&self, name: &str) -> Option<Variable> {
        match self.variables.get(name) {
            Some(var) => Some(var.clone()),
            None => {
                if let Some(environment) = self.environment.as_ref() {
                    environment.borrow().get_var(name)
                } else {
                    None
                }
            }
        }
    }

    pub fn get_value(&self, name: &str) -> Option<Literal> {
        if let Some(var) = self.get_var(name) {
            var.value
        } else {
            None
        }
    }

    pub fn register_function(&mut self, name: &str, function: FunctionVariant) {
        self.functions.insert(name.to_string(), function);
    }

    pub fn register_function_in_namespace(
        &mut self,
        namespace_name: &str,
        name: &str,
        function: FunctionVariant,
    ) {
        let namespace_name = namespace_name.to_string();
        if let Some(namespace) = self.namespaces.get_mut(&namespace_name) {
            namespace.register_function(name, function);
        } else {
            let mut namespace: Namespace = Default::default();
            namespace.register_function(name, function);
            self.namespaces.insert(namespace_name, namespace);
        }
    }

    pub fn import_namespace(&mut self, name: &str) -> Result<(), String> {
        let namespace = self
            .namespaces
            .get_mut(&name.to_string())
            .ok_or(format!("Namespace with name: {:?} not found", name))?
            .clone();
        for (name, function) in namespace.functions().into_iter() {
            self.register_function(&name, function.clone());
        }
        Ok(())
    }

    pub fn register_namespace(&mut self, name: &str, namespace: Namespace) {
        self.namespaces.insert(name.to_string(), namespace);
    }

    pub fn get_function(&self, name: &str) -> Option<FunctionVariant> {
        if let Some(func) = self.functions.get(name) {
            return Some(func.clone());
        }
        if let Some(parent) = self.environment.as_ref() {
            return parent.borrow().get_function(name);
        }
        None
    }

    pub fn get_all_vars(&self) -> Vec<(String, Variable)> {
        let mut vars = vec![];
        vars.extend(
            self.variables
                .iter()
                .map(|(key, var)| (key.clone(), var.clone())),
        );
        if let Some(environment) = self.environment.as_ref() {
            vars.extend(environment.borrow().get_all_vars().iter().cloned())
        }
        vars
    }

    pub fn set_kill_flag(&mut self, kill_flag: Arc<AtomicBool>) {
        self.kill_flag = kill_flag
    }
}

use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::{cell::RefCell, rc::Rc};

use indexmap::IndexMap;
use kumir_lang::ast::Namespace;
use kumir_lang::{ast::Literal, lexer::TypeDefinition};
use kumir_lang::{
    ast::{Environment, NativeFunction},
    interpreter::Interpreter,
};
use log::info;

use crate::{Lang, Runtime, RuntimeRequirements};

macro_rules! register {
    ($interpreter:expr, $requirements:expr, $method_name:ident, $func_name:expr) => {
        let req = $requirements.clone();
        $interpreter.register_native_function(
            $func_name,
            NativeFunction {
                params: IndexMap::new(),
                return_type: None,
                native_function: Rc::new(RefCell::new(move |_: &Rc<RefCell<Environment>>| {
                    let res = req.$method_name();
                    match res {
                        Ok(_) => Ok(None),
                        Err(arg) => Err(arg),
                    }
                })),
            },
        );
    };
    ($interpreter:expr, $requirements:expr, $method_name:ident, $func_name:expr, $return_type:ident) => {
        let req = $requirements.clone();
        $interpreter.register_native_function(
            $func_name,
            NativeFunction {
                params: IndexMap::new(),
                return_type: Some(TypeDefinition::$return_type),
                native_function: Rc::new(RefCell::new(move |_: &Rc<RefCell<Environment>>| {
                    let res = req.$method_name();
                    match res {
                        Ok(arg) => Ok(Some(Literal::$return_type(arg.unwrap()))),
                        Err(arg) => Err(arg),
                    }
                })),
            },
        )
    };
}

pub struct KumirLangRuntime {
    requirements: RuntimeRequirements,
    interpreter: Interpreter,
}

impl Runtime for KumirLangRuntime {
    fn init(
        requirements: RuntimeRequirements,
        _: Lang,
        code: String,
        kill_flag: Arc<AtomicBool>,
    ) -> Result<Self, String> {
        info!("Initializing KuMir lang runtime");
        let mut interpreter = Interpreter::new_from_string(&code, kill_flag)?;
        interpreter.register_namespace("Робот", {
            let mut namespace: Namespace = Default::default();
            namespace.register_native_function(
                "test",
                NativeFunction {
                    params: IndexMap::new(),
                    return_type: Some(TypeDefinition::Bool),
                    native_function: Rc::new(RefCell::new(move |_: &Rc<RefCell<Environment>>| {
                        Ok(Some(Literal::Bool(true)))
                    })),
                },
            );
            namespace
        });
        register!(&mut interpreter, requirements, move_up, "вверх");
        register!(&mut interpreter, requirements, move_down, "вниз");
        register!(&mut interpreter, requirements, move_left, "влево");
        register!(&mut interpreter, requirements, move_right, "вправо");
        register!(&mut interpreter, requirements, paint, "закрасить");
        register!(
            &mut interpreter,
            requirements,
            free_left,
            "слева свободно",
            Bool
        );
        register!(
            &mut interpreter,
            requirements,
            free_right,
            "справа свободно",
            Bool
        );
        register!(
            &mut interpreter,
            requirements,
            free_above,
            "сверху свободно",
            Bool
        );
        register!(
            &mut interpreter,
            requirements,
            free_below,
            "снизу свободно",
            Bool
        );
        register!(
            &mut interpreter,
            requirements,
            wall_left,
            "слева стена",
            Bool
        );
        register!(
            &mut interpreter,
            requirements,
            wall_right,
            "справа стена",
            Bool
        );
        register!(
            &mut interpreter,
            requirements,
            wall_above,
            "стена сверху",
            Bool
        );
        register!(
            &mut interpreter,
            requirements,
            wall_below,
            "стена снизу",
            Bool
        );
        register!(
            &mut interpreter,
            requirements,
            colored,
            "клетка закрашена",
            Bool
        );
        register!(
            &mut interpreter,
            requirements,
            not_colored,
            "клетка чистая",
            Bool
        );
        Ok(Self {
            requirements,
            interpreter,
        })
    }

    fn run(&mut self) -> Result<(), String> {
        self.requirements
            .println("Hello from runtime in requirements");
        self.interpreter.run()?;
        Ok(())
    }
}

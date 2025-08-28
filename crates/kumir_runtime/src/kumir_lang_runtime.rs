use std::{cell::RefCell, rc::Rc};

use indexmap::IndexMap;
use kumir_lang::{
    ast::{Environment, NativeFunction},
    interpreter::Interpreter,
};
use log::info;

use crate::{Lang, Runtime, RuntimeRequirements};

macro_rules! register {
    ($interpreter:expr, $requirements:expr, $method_name:ident, $func_name:expr, $return_type:expr) => {
        let req = $requirements.clone();
        $interpreter.register_native_function(
            $func_name,
            NativeFunction {
                params: IndexMap::new(),
                return_type: $return_type,
                native_function: Rc::new(RefCell::new(move |_: &Rc<RefCell<Environment>>| {
                    req.$method_name();
                    Ok(None)
                })),
            },
        );
    };
}

pub struct KumirLangRuntime {
    requirements: RuntimeRequirements,
    interpreter: Interpreter,
}

impl Runtime for KumirLangRuntime {
    fn init(requirements: RuntimeRequirements, _: Lang, code: String) -> Result<Self, String> {
        info!("Initializing KuMir lang runtime");
        let mut interpreter = Interpreter::new_from_string(&code)?;
        register!(&mut interpreter, requirements, move_up, "вверх", None);
        register!(&mut interpreter, requirements, move_down, "вниз", None);
        register!(&mut interpreter, requirements, move_left, "влево", None);
        register!(&mut interpreter, requirements, move_right, "вправо", None);
        register!(&mut interpreter, requirements, paint, "закрасить", None);
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

use std::{cell::RefCell, rc::Rc};

use indexmap::IndexMap;
use kumir_lang::{
    ast::{Environment, NativeFunction},
    interpreter::Interpreter,
};
use log::info;

use crate::{Lang, Runtime, RuntimeRequirements};

pub struct KumirLangRuntime {
    requirements: RuntimeRequirements,
    interpreter: Interpreter,
}

impl Runtime for KumirLangRuntime {
    fn init(requirements: RuntimeRequirements, _: Lang, code: String) -> Result<Self, String> {
        info!("Initializing KuMir lang runtime");
        let mut interpreter = Interpreter::new_from_string(&code)?;
        register_move_up(&mut interpreter, requirements.clone());
        register_move_down(&mut interpreter, requirements.clone());
        register_move_left(&mut interpreter, requirements.clone());
        register_move_right(&mut interpreter, requirements.clone());
        register_paint(&mut interpreter, requirements.clone());
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

fn register_move_up(interpreter: &mut Interpreter, requirements: RuntimeRequirements) {
    interpreter.register_native_function(
        "вверх",
        NativeFunction {
            params: IndexMap::new(),
            return_type: None,
            native_function: Rc::new(RefCell::new(move |_: &Rc<RefCell<Environment>>| {
                requirements.move_up();
                Ok(None)
            })),
        },
    );
}

fn register_move_down(interpreter: &mut Interpreter, requirements: RuntimeRequirements) {
    interpreter.register_native_function(
        "вниз",
        NativeFunction {
            params: IndexMap::new(),
            return_type: None,
            native_function: Rc::new(RefCell::new(move |_: &Rc<RefCell<Environment>>| {
                requirements.move_down();
                Ok(None)
            })),
        },
    );
}

fn register_move_left(interpreter: &mut Interpreter, requirements: RuntimeRequirements) {
    interpreter.register_native_function(
        "влево",
        NativeFunction {
            params: IndexMap::new(),
            return_type: None,
            native_function: Rc::new(RefCell::new(move |_: &Rc<RefCell<Environment>>| {
                requirements.move_left();
                Ok(None)
            })),
        },
    );
}

fn register_move_right(interpreter: &mut Interpreter, requirements: RuntimeRequirements) {
    interpreter.register_native_function(
        "вправо",
        NativeFunction {
            params: IndexMap::new(),
            return_type: None,
            native_function: Rc::new(RefCell::new(move |_: &Rc<RefCell<Environment>>| {
                requirements.move_right();
                Ok(None)
            })),
        },
    );
}

fn register_paint(interpreter: &mut Interpreter, requirements: RuntimeRequirements) {
    interpreter.register_native_function(
        "закрасить",
        NativeFunction {
            params: IndexMap::new(),
            return_type: None,
            native_function: Rc::new(RefCell::new(move |_: &Rc<RefCell<Environment>>| {
                requirements.paint();
                Ok(None)
            })),
        },
    );
}

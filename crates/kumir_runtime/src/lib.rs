pub mod console_runtime_requirements;
pub mod kumir_lang_runtime;
pub mod python_runtime;
use std::fmt;
use std::sync::Arc;

use log::info;
use rustpython_vm::PyRef;

use crate::kumir_lang_runtime::KumirLangRuntime;
use crate::python_runtime::PythonRuntime;

#[derive(PartialEq, Clone)]
pub enum Lang {
    Kumir,
    Python,
}

impl fmt::Display for Lang {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Lang::Python => write!(f, "Python"),
            Lang::Kumir => write!(f, "КуМир"),
        }
    }
}

pub enum Target {
    KumirLang(KumirLangRuntime),
    Python(PythonRuntime),
    #[cfg(not(target_arch = "wasm32"))]
    Wasmtime(),
}

impl Runtime for Target {
    fn init(requirements: RuntimeRequirements, lang: Lang, code: String) -> Result<Self, String> {
        match lang {
            Lang::Kumir => Ok(Target::KumirLang(KumirLangRuntime::init(
                requirements,
                lang,
                code,
            )?)),
            Lang::Python => Ok(Target::Python(PythonRuntime::init(
                requirements,
                lang,
                code,
            )?)),
        }
    }

    fn run(&mut self) -> Result<(), String> {
        match self {
            Target::KumirLang(kumir_lang_runtime) => kumir_lang_runtime.run(),
            Target::Python(python_runtime) => python_runtime.run(),
            Target::Wasmtime() => todo!(),
        }
    }
}

pub type RuntimeRequirements = Arc<dyn RuntimeRequirementsTrait + 'static>;

pub trait RuntimeRequirementsTrait: RobotRequirements + Send + Sync {
    fn println(&self, message: &str);
}

pub trait RobotRequirements {
    fn move_up(&self);
    fn move_down(&self);
    fn move_left(&self);
    fn move_right(&self);
    fn paint(&self);
}

pub trait Runtime {
    fn init(requirements: RuntimeRequirements, lang: Lang, code: String) -> Result<Self, String>
    where
        Self: Sized;

    fn run(&mut self) -> Result<(), String>;
}

pub mod console_runtime_requirements;
pub mod kumir_lang_runtime;
pub mod python_runtime;
use std::fmt;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, mpsc};

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
    fn init(
        requirements: RuntimeRequirements,
        lang: Lang,
        code: String,
        kill_flag: Arc<AtomicBool>,
    ) -> Result<Self, String> {
        match lang {
            Lang::Kumir => Ok(Target::KumirLang(KumirLangRuntime::init(
                requirements,
                lang,
                code,
                kill_flag,
            )?)),
            Lang::Python => Ok(Target::Python(PythonRuntime::init(
                requirements,
                lang,
                code,
                kill_flag,
            )?)),
        }
    }

    fn run(&mut self) -> Result<(), String> {
        match self {
            Target::KumirLang(kumir_lang_runtime) => kumir_lang_runtime.run(),
            Target::Python(python_runtime) => python_runtime.run(),
            #[cfg(not(target_arch = "wasm32"))]
            Target::Wasmtime() => todo!(),
        }
    }
}

pub type RuntimeRequirements = Arc<dyn RuntimeRequirementsTrait + 'static>;
pub type FuncResult<T> = Result<Option<T>, String>;

pub trait RuntimeRequirementsTrait: RobotRequirements + Send + Sync {
    fn println(&self, message: &str);
}

pub trait RobotRequirements {
    fn move_up(&self) -> FuncResult<()>;
    fn move_down(&self) -> FuncResult<()>;
    fn move_left(&self) -> FuncResult<()>;
    fn move_right(&self) -> FuncResult<()>;
    fn paint(&self) -> FuncResult<()>;
    fn free_right(&self) -> FuncResult<bool>;
    fn free_left(&self) -> FuncResult<bool>;
    fn free_above(&self) -> FuncResult<bool>;
    fn free_below(&self) -> FuncResult<bool>;
    fn wall_left(&self) -> FuncResult<bool>;
    fn wall_right(&self) -> FuncResult<bool>;
    fn wall_above(&self) -> FuncResult<bool>;
    fn wall_below(&self) -> FuncResult<bool>;
    fn colored(&self) -> FuncResult<bool>;
    fn not_colored(&self) -> FuncResult<bool>;
}

pub trait Runtime {
    fn init(
        requirements: RuntimeRequirements,
        lang: Lang,
        code: String,
        kill_flag: Arc<AtomicBool>,
    ) -> Result<Self, String>
    where
        Self: Sized;

    fn run(&mut self) -> Result<(), String>;
}

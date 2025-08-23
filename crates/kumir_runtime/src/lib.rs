pub mod console_runtime_requirements;
pub mod kumir_lang_runtime;
use std::sync::Arc;

use log::info;

use crate::kumir_lang_runtime::KumirLangRuntime;

pub enum Lang {
    Kumir,
    Python,
}

pub enum Target {
    KumirLang(KumirLangRuntime),
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
            Lang::Python => todo!(),
        }
    }

    fn run(&mut self) -> Result<(), String> {
        match self {
            Target::KumirLang(kumir_lang_runtime) => kumir_lang_runtime.run(),
            Target::Wasmtime() => todo!(),
        }
    }
}

pub type RuntimeRequirements = Arc<dyn RuntimeRequirementsTrait + 'static>;

pub trait RuntimeRequirementsTrait: RobotRequirements {
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

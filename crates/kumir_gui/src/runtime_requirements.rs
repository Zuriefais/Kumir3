use kumir_runtime::{FuncResult, RobotRequirements, RuntimeRequirementsTrait};
use log::info;
use std::time::Duration;
use wasm_thread as thread;

use crate::kumir_state::Modes;

macro_rules! call_method_in_enum {
    ($object:expr, $class:ident::$variant:ident, $method_name:ident) => {
        match $object {
            $class::$variant(executor) => {
                let mut obj = executor.lock().unwrap();
                obj.$method_name()
            }
            _ => Err("Выбранный исполнитель отличается от загружаемого".to_string()),
        }
    };
    ($object:expr, $class:ident::$variant:ident, $method_name:ident, $($arg:expr),+) => {
        match $object {
            $class::$variant(executor) => {
                let mut obj = executor.lock().unwrap();
                obj.$method_name($($arg),+)
            }
            _ => Err("Выбранный исполнитель отличается от загружаемого".to_string()),
        }
    };
}

pub struct GuiRuntimeRequirements {
    pub mode: Modes,
    pub sleep_duration: Duration,
}

impl RuntimeRequirementsTrait for GuiRuntimeRequirements {
    fn println(&self, message: &str) {
        info!("{message}")
    }
}

impl RobotRequirements for GuiRuntimeRequirements {
    fn move_up(&self) -> FuncResult<()> {
        let res = call_method_in_enum!(self.mode.clone(), Modes::Robot, move_up);
        thread::sleep(self.sleep_duration);
        res
    }

    fn move_down(&self) -> FuncResult<()> {
        let res = call_method_in_enum!(self.mode.clone(), Modes::Robot, move_down);
        thread::sleep(self.sleep_duration);
        res
    }

    fn move_left(&self) -> FuncResult<()> {
        let res = call_method_in_enum!(self.mode.clone(), Modes::Robot, move_left);
        thread::sleep(self.sleep_duration);
        res
    }

    fn move_right(&self) -> FuncResult<()> {
        let res = call_method_in_enum!(self.mode.clone(), Modes::Robot, move_right);
        thread::sleep(self.sleep_duration);
        res
    }

    fn paint(&self) -> FuncResult<()> {
        thread::sleep(self.sleep_duration);
        call_method_in_enum!(self.mode.clone(), Modes::Robot, paint)
    }

    fn free_right(&self) -> FuncResult<bool> {
        call_method_in_enum!(self.mode.clone(), Modes::Robot, free_right)
    }

    fn free_left(&self) -> FuncResult<bool> {
        call_method_in_enum!(self.mode.clone(), Modes::Robot, free_left)
    }

    fn free_above(&self) -> FuncResult<bool> {
        call_method_in_enum!(self.mode.clone(), Modes::Robot, free_above)
    }

    fn free_below(&self) -> FuncResult<bool> {
        call_method_in_enum!(self.mode.clone(), Modes::Robot, free_below)
    }

    fn wall_right(&self) -> FuncResult<bool> {
        call_method_in_enum!(self.mode.clone(), Modes::Robot, wall_right)
    }

    fn wall_left(&self) -> FuncResult<bool> {
        call_method_in_enum!(self.mode.clone(), Modes::Robot, wall_left)
    }

    fn wall_above(&self) -> FuncResult<bool> {
        call_method_in_enum!(self.mode.clone(), Modes::Robot, wall_above)
    }

    fn wall_below(&self) -> FuncResult<bool> {
        call_method_in_enum!(self.mode.clone(), Modes::Robot, wall_below)
    }

    fn colored(&self) -> FuncResult<bool> {
        call_method_in_enum!(self.mode.clone(), Modes::Robot, colored)
    }

    fn not_colored(&self) -> FuncResult<bool> {
        call_method_in_enum!(self.mode.clone(), Modes::Robot, not_colored)
    }
}

use std::sync::Mutex;

use kumir_runtime::{RobotRequirements, RuntimeRequirementsTrait};
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
            _ => false,
        }
    };
    ($object:expr, $class:ident::$variant:ident, $method_name:ident, $($arg:expr),+) => {
        match $object {
            $class::$variant(executor) => {
                let mut obj = executor.lock().unwrap();
                obj.$method_name($($arg),+)
            }
            _ => false,
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
    fn move_up(&self) {
        call_method_in_enum!(self.mode.clone(), Modes::Robot, move_robot, 0, -1);
        thread::sleep(self.sleep_duration);
    }

    fn move_down(&self) {
        call_method_in_enum!(self.mode.clone(), Modes::Robot, move_robot, 0, 1);
        thread::sleep(self.sleep_duration);
    }

    fn move_left(&self) {
        call_method_in_enum!(self.mode.clone(), Modes::Robot, move_robot, -1, 0);
        thread::sleep(self.sleep_duration);
    }

    fn move_right(&self) {
        call_method_in_enum!(self.mode.clone(), Modes::Robot, move_robot, 1, 0);
        thread::sleep(self.sleep_duration);
    }

    fn paint(&self) {
        call_method_in_enum!(self.mode.clone(), Modes::Robot, paint);
        thread::sleep(self.sleep_duration);
    }

    fn free_right(&self) -> bool {
        call_method_in_enum!(self.mode.clone(), Modes::Robot, free_right)
    }

    fn free_left(&self) -> bool {
        call_method_in_enum!(self.mode.clone(), Modes::Robot, free_left)
    }

    fn free_above(&self) -> bool {
        call_method_in_enum!(self.mode.clone(), Modes::Robot, free_above)
    }

    fn free_below(&self) -> bool {
        call_method_in_enum!(self.mode.clone(), Modes::Robot, free_below)
    }

    fn wall_right(&self) -> bool {
        call_method_in_enum!(self.mode.clone(), Modes::Robot, wall_right)
    }

    fn wall_left(&self) -> bool {
        call_method_in_enum!(self.mode.clone(), Modes::Robot, wall_left)
    }

    fn wall_above(&self) -> bool {
        call_method_in_enum!(self.mode.clone(), Modes::Robot, wall_above)
    }

    fn wall_below(&self) -> bool {
        call_method_in_enum!(self.mode.clone(), Modes::Robot, wall_below)
    }

    fn colored(&self) -> bool {
        call_method_in_enum!(self.mode.clone(), Modes::Robot, colored)
    }

    fn not_colored(&self) -> bool {
        call_method_in_enum!(self.mode.clone(), Modes::Robot, not_colored)
    }
}

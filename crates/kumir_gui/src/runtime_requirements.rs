use std::sync::Mutex;

use kumir_runtime::{RobotRequirements, RuntimeRequirementsTrait};
use log::info;
use std::time::Duration;
use wasm_thread as thread;

use crate::kumir_state::Modes;

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
        match self.mode.clone() {
            Modes::Robot(executor) => {
                let mut robot = executor.lock().unwrap();
                robot.move_robot(0, -1);
            }
            _ => (),
        }

        thread::sleep(self.sleep_duration);
    }

    fn move_down(&self) {
        match self.mode.clone() {
            Modes::Robot(executor) => {
                let mut robot = executor.lock().unwrap();
                robot.move_robot(0, 1);
            }
            _ => (),
        }

        thread::sleep(self.sleep_duration);
    }

    fn move_left(&self) {
        match self.mode.clone() {
            Modes::Robot(executor) => {
                let mut robot = executor.lock().unwrap();
                robot.move_robot(-1, 0);
            }
            _ => (),
        }

        thread::sleep(self.sleep_duration);
    }

    fn move_right(&self) {
        match self.mode.clone() {
            Modes::Robot(executor) => {
                let mut robot = executor.lock().unwrap();
                robot.move_robot(1, 0);
            }
            _ => (),
        }

        thread::sleep(self.sleep_duration);
    }

    fn paint(&self) {
        match self.mode.clone() {
            Modes::Robot(executor) => {
                let mut robot = executor.lock().unwrap();
                robot.paint();
            }
            _ => (),
        }

        thread::sleep(self.sleep_duration);
    }
}

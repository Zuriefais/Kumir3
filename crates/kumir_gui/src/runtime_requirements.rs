use std::sync::Mutex;

use kumir_runtime::{RobotRequirements, RuntimeRequirementsTrait};
use log::info;

use crate::executors::robot::Robot;

pub struct GuiRuntimeRequirements {
    pub robot: Mutex<Robot>,
}

impl RuntimeRequirementsTrait for GuiRuntimeRequirements {
    fn println(&self, message: &str) {
        info!("{message}")
    }
}

impl RobotRequirements for GuiRuntimeRequirements {
    fn move_up(&self) {
        let mut robot = self.robot.lock().unwrap();
        robot.move_robot(0, 1);
    }

    fn move_down(&self) {
        let mut robot = self.robot.lock().unwrap();
        robot.move_robot(0, -1);
    }

    fn move_left(&self) {
        let mut robot = self.robot.lock().unwrap();
        robot.move_robot(-1, 0);
    }

    fn move_right(&self) {
        let mut robot = self.robot.lock().unwrap();
        robot.move_robot(1, 0);
    }

    fn paint(&self) {
        let mut robot = self.robot.lock().unwrap();
        todo!("Impl Paint in Robot obj")
    }
}

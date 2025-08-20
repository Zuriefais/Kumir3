use log::info;

use crate::{RobotRequirements, RuntimeRequirementsTrait};

pub struct ConsoleRuntimeRequirements;

impl RuntimeRequirementsTrait for ConsoleRuntimeRequirements {
    fn println(&self, message: &str) {
        info!("{}", message)
    }
}

impl RobotRequirements for ConsoleRuntimeRequirements {
    fn move_up(&self) {
        info!("Move up")
    }

    fn move_down(&self) {
        info!("Move down")
    }

    fn move_left(&self) {
        info!("Move left")
    }

    fn move_right(&self) {
        info!("Move right")
    }

    fn paint(&self) {
        info!("Painting")
    }
}

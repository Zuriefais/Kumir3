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

    fn free_above(&self) -> bool {
        info!("Free above");
        true
    }

    fn free_below(&self) -> bool {
        info!("Free below");
        true
    }

    fn free_left(&self) -> bool {
        info!("Free left");
        true
    }

    fn free_right(&self) -> bool {
        info!("Free right");
        true
    }

    fn wall_above(&self) -> bool {
        info!("Wall above");
        false
    }

    fn wall_below(&self) -> bool {
        info!("Wall below");
        false
    }

    fn wall_left(&self) -> bool {
        info!("Wall left");
        false
    }

    fn wall_right(&self) -> bool {
        info!("Wall right");
        false
    }

    fn colored(&self) -> bool {
        info!("Colored");
        false
    }

    fn not_colored(&self) -> bool {
        info!("Not colored");
        true
    }
}

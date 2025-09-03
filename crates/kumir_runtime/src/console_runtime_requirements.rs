use log::info;

use crate::{FuncResult, RobotRequirements, RuntimeRequirementsTrait};

pub struct ConsoleRuntimeRequirements;

impl RuntimeRequirementsTrait for ConsoleRuntimeRequirements {
    fn println(&self, message: &str) {
        info!("{}", message)
    }
}

impl RobotRequirements for ConsoleRuntimeRequirements {
    fn move_up(&self) -> FuncResult<()> {
        info!("Move up");
        Ok(None)
    }

    fn move_down(&self) -> FuncResult<()> {
        info!("Move down");
        Ok(None)
    }

    fn move_left(&self) -> FuncResult<()> {
        info!("Move left");
        Ok(None)
    }

    fn move_right(&self) -> FuncResult<()> {
        info!("Move right");
        Ok(None)
    }

    fn paint(&self) -> FuncResult<()> {
        info!("Painting");
        Ok(None)
    }

    fn free_above(&self) -> FuncResult<bool> {
        info!("Free above");
        Ok(Some(true))
    }

    fn free_below(&self) -> FuncResult<bool> {
        info!("Free below");
        Ok(Some(true))
    }

    fn free_left(&self) -> FuncResult<bool> {
        info!("Free left");
        Ok(Some(true))
    }

    fn free_right(&self) -> FuncResult<bool> {
        info!("Free right");
        Ok(Some(true))
    }

    fn wall_above(&self) -> FuncResult<bool> {
        info!("Wall above");
        Ok(Some(false))
    }

    fn wall_below(&self) -> FuncResult<bool> {
        info!("Wall below");
        Ok(Some(false))
    }

    fn wall_left(&self) -> FuncResult<bool> {
        info!("Wall left");
        Ok(Some(false))
    }

    fn wall_right(&self) -> FuncResult<bool> {
        info!("Wall right");
        Ok(Some(false))
    }

    fn colored(&self) -> FuncResult<bool> {
        info!("Colored");
        Ok(Some(false))
    }

    fn not_colored(&self) -> FuncResult<bool> {
        info!("Not colored");
        Ok(Some(true))
    }
}

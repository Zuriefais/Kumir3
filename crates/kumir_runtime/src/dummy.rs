use log::info;

pub struct DummyRuntimeRequirements;

impl RuntimeRequirements for DummyRuntimeRequirements {
    fn println(&self, message: &str) {
        info!("{}", message);
    }
}

impl RobotRequirements for DummyRuntimeRequirements {
    fn move_up(&self) {
        info!("Moving up");
    }

    fn move_down(&self) {
        info!("Moving down");
    }

    fn move_left(&self) {
        info!("Moving left");
    }

    fn move_right(&self) {
        info!("Moving right");
    }

    fn paint(&self) {
        info!("Painting");
    }
}

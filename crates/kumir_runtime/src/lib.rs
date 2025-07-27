use log::info;

pub enum Target {
    Interpreter(Interpreter),
    Dummy(DummyRuntime),
    #[cfg(not(target_arch = "wasm32"))]
    Wasmtime(WasmtimeTarget),
}

#[cfg(not(target_arch = "wasm32"))]
pub enum WasmtimeTarget {}

pub trait RuntimeRequirements: RobotRequirements {
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
    fn init(requirements: Box<dyn RuntimeRequirements + 'static>) -> Self;

    fn run(&self);
}

pub struct DummyRuntime {
    requirements: Box<dyn RuntimeRequirements>,
}

impl Runtime for DummyRuntime {
    fn init(requirements: Box<dyn RuntimeRequirements + 'static>) -> Self {
        Self { requirements }
    }

    fn run(&self) {
        self.requirements.move_up();
    }
}

pub struct Interpreter {
    requirements: Box<dyn RuntimeRequirements>,
}

impl Runtime for Interpreter {
    fn init(requirements: Box<dyn RuntimeRequirements + 'static>) -> Self {
        Self { requirements }
    }

    fn run(&self) {
        self.requirements.move_up();
    }
}

pub struct DummyRuntimeRequirements {}
impl RuntimeRequirements for DummyRuntimeRequirements {
    fn println(&self, message: &str) {
        info!("{message}");
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

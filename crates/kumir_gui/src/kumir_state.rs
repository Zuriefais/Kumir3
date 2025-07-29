use crate::executors::robot::Robot;
use std::fmt;
use std::sync::{Arc, Mutex};
use vello::Scene;

#[derive(PartialEq, Eq, Clone)]
pub enum Modes {
    None,
    Kuznechik,
    Vodolei,
    Cherepaha,
    Chertezhnik,
    Robot,
}

impl fmt::Display for Modes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Modes::None => write!(f, "Не выбрано"),
            Modes::Kuznechik => write!(f, "Кузнечик"),
            Modes::Vodolei => write!(f, "Водолей"),
            Modes::Robot => write!(f, "Робот"),
            Modes::Chertezhnik => write!(f, "Чертежник"),
            Modes::Cherepaha => write!(f, "Черепаха"),
        }
    }
}

pub struct KumirState {
    scene: Arc<Mutex<Scene>>,
    width: u32,
    height: u32,
}

impl KumirState {
    pub fn new(scene: Arc<Mutex<Scene>>, width: u32, height: u32) -> KumirState {
        KumirState {
            scene: scene,
            width: width,
            height: height,
        }
    }

    pub fn add_shapes_to_scene(&mut self) {
        let rob = Robot::new(9, 9, 100.0);
        rob.draw_field(&mut self.scene.lock().unwrap());
    }
}

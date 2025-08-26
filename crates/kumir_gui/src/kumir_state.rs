use crate::executors::robot::Robot;
use crate::executors::{Executor, NoneSelected};
use egui::Pos2;

use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard};

use vello::Scene;
use vello::peniko::Color;

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

#[derive(Debug, Clone)]
pub struct ModesStored {
    pub robot: Arc<Mutex<dyn Executor>>,
    pub none: Arc<Mutex<dyn Executor>>,
}

pub enum VisualMode {
    Dark,
    Light,
}

pub struct KumirState {
    pub scene: Arc<Mutex<Scene>>,
    pub width: f64,
    pub height: f64,
    pub selected_mode: Modes,
    pub modes: ModesStored,
    pub visual_mode: VisualMode,
    pub min_point: Pos2,
}

impl KumirState {
    pub fn new(scene: Arc<Mutex<Scene>>, width: f64, height: f64) -> KumirState {
        let rob = Arc::new(Mutex::new(Robot::new(
            9,
            9,
            100.0,
            width / 2.0,
            height / 2.0,
        )));
        let none = Arc::new(Mutex::new(NoneSelected::new()));
        KumirState {
            scene: scene,
            width: width,
            height: height,
            selected_mode: Modes::None,
            modes: ModesStored {
                robot: rob,
                none: none,
            },
            visual_mode: VisualMode::Dark,
            min_point: Pos2::new(10.0, 85.0),
        }
    }

    pub fn current_mode(&self) -> MutexGuard<'_, dyn Executor> {
        match self.selected_mode {
            Modes::Robot => self.modes.robot.lock().unwrap(),
            _ => self.modes.none.lock().unwrap(),
        }
    }

    pub fn add_shapes_to_scene(&mut self) {
        self.current_mode()
            .draw_field(&mut self.scene.lock().unwrap());
    }

    pub fn update_transform(&mut self, width: f64, height: f64) {
        self.modes
            .robot
            .lock()
            .unwrap()
            .update_transform(width, height);
    }

    pub fn update_min_point(&mut self, pos: Pos2) {
        if self.min_point != pos {
            self.min_point = pos;
        }
    }

    pub fn change_scale(&mut self, scale_delta: f64) {
        self.current_mode().change_scale(scale_delta);
    }

    pub fn get_scale(&self) -> f64 {
        self.current_mode().get_scale()
    }

    pub fn base_color(&self) -> Color {
        self.current_mode().base_color()
    }

    pub fn hover(&self, pos: Option<Pos2>, pixels_per_point: f32) {
        if pos == None {
            return;
        }

        self.current_mode()
            .hovered((pos.unwrap() - self.min_point).to_pos2(), pixels_per_point);
    }

    pub fn click(&self) {
        self.current_mode().clicked();
    }
}

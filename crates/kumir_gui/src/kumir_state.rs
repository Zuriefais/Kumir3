use crate::executors::robot::{ColumnsMode, Robot, RobotEditingState, RowsMode};
use egui::Pos2;
use log::info;
use std::fmt;
use std::sync::{Arc, Mutex};
use std::time;
use vello::Scene;
use vello::peniko::Color;
use wasm_thread as thread;

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

pub struct ModesStored {
    pub robot: Arc<Mutex<Robot>>,
}

pub struct EditingStates {
    pub robot: RobotEditingState,
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
    pub editing_states: EditingStates,
    pub visual_mode: VisualMode,
    pub min_point: Pos2,
}

impl KumirState {
    pub fn new(scene: Arc<Mutex<Scene>>, width: f64, height: f64) -> KumirState {
        KumirState {
            scene: scene,
            width: width,
            height: height,
            selected_mode: Modes::None,
            modes: ModesStored {
                robot: Arc::new(Mutex::new(Robot::new(
                    9,
                    9,
                    100.0,
                    width / 2.0,
                    height / 2.0,
                ))),
            },
            editing_states: EditingStates {
                robot: RobotEditingState {
                    deleting_rows_mode: RowsMode::FromDown,
                    deleting_columns_mode: ColumnsMode::FromRight,
                },
            },
            visual_mode: VisualMode::Dark,
            min_point: Pos2::new(10.0, 85.0),
        }
    }

    pub fn add_shapes_to_scene(&mut self) {
        match self.selected_mode {
            // Modes::None => todo!(),
            // Modes::Kuznechik => todo!(),
            // Modes::Vodolei => todo!(),
            // Modes::Cherepaha => todo!(),
            // Modes::Chertezhnik => todo!(),
            Modes::Robot => self
                .modes
                .robot
                .lock()
                .unwrap()
                .draw_field(&mut self.scene.lock().unwrap()),
            _ => (),
        }
    }

    pub fn update_transform(&mut self, width: f64, height: f64) {
        self.modes
            .robot
            .lock()
            .unwrap()
            .update_centers(width, height);
    }

    pub fn run(&mut self) {
        let rob = Arc::clone(&self.modes.robot);
        info!("run");

        thread::spawn(move || {
            rob.lock().unwrap().move_right();
            thread::sleep(time::Duration::from_millis(1000));
            rob.lock().unwrap().move_left();
        });
    }

    pub fn change_offset(&mut self, o: f32, i: f32) {
        match self.selected_mode {
            _ => (),
        }
    }

    pub fn update_min_point(&mut self, pos: Pos2) {
        if self.min_point != pos {
            self.min_point = pos;
        }
    }

    pub fn change_scale(&mut self, scale_delta: f64) {
        match self.selected_mode {
            Modes::Robot => self.modes.robot.lock().unwrap().change_scale(scale_delta),
            _ => (),
        }
    }

    pub fn get_scale(&self) -> f64 {
        match self.selected_mode {
            Modes::Robot => self.modes.robot.lock().unwrap().get_scale(),
            _ => 1.0,
        }
    }

    pub fn base_color(&self) -> Color {
        match self.selected_mode {
            Modes::Robot => self.modes.robot.lock().unwrap().base_color(),
            _ => Color::from_rgb8(0, 0, 0),
        }
    }

    pub fn hover(&self, pos: Option<Pos2>) {
        if pos == None {
            return;
        }

        match self.selected_mode {
            Modes::Robot => self
                .modes
                .robot
                .lock()
                .unwrap()
                .hovered((pos.unwrap() - self.min_point).to_pos2()),
            _ => (),
        }
    }

    pub fn click(&self) {
        match self.selected_mode {
            Modes::Robot => self.modes.robot.lock().unwrap().clicked(),
            _ => (),
        }
    }
}

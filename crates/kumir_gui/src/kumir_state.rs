use crate::executors::robot::{ColumnsMode, Robot, RobotEditingState, RowsMode};
use log::info;
use std::fmt;
use std::sync::{Arc, Mutex};
use std::time;
use vello::Scene;
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

pub struct KumirState {
    pub scene: Arc<Mutex<Scene>>,
    pub width: u32,
    pub height: u32,
    pub selected_mode: Modes,
    pub modes: ModesStored,
    pub editing_states: EditingStates,
}

impl KumirState {
    pub fn new(scene: Arc<Mutex<Scene>>, width: u32, height: u32) -> KumirState {
        KumirState {
            scene: scene,
            width: width,
            height: height,
            selected_mode: Modes::None,
            modes: ModesStored {
                robot: Arc::new(Mutex::new(Robot::new(9, 9, 100.0))),
            },
            editing_states: EditingStates {
                robot: RobotEditingState {
                    deleting_rows_mode: RowsMode::FromDown,
                    deleting_columns_mode: ColumnsMode::FromRight,
                },
            },
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
            Modes::Robot => self
                .modes
                .robot
                .lock()
                .unwrap()
                .change_offset(o as f64, i as f64),
            _ => (),
        }
    }

    pub fn change_scale(&mut self, scale_delta: f64) {
        match self.selected_mode {
            Modes::Robot => self.modes.robot.lock().unwrap().change_scale(scale_delta),
            _ => (),
        }
    }
}

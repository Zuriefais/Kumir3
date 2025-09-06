use crate::executors::robot::Robot;
use crate::executors::{Executor, NoneSelected};
use egui::{Pos2, Vec2};

use std::fmt;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use vello::Scene;
use vello::peniko::Color;

#[derive(Clone, Debug)]
pub enum Modes {
    None(Arc<Mutex<NoneSelected>>),
    Kuznechik(Arc<Mutex<NoneSelected>>),
    Vodolei(Arc<Mutex<NoneSelected>>),
    Cherepaha(Arc<Mutex<NoneSelected>>),
    Chertezhnik(Arc<Mutex<NoneSelected>>),
    Robot(Arc<Mutex<Robot>>),
}

impl Executor for Modes {
    fn clear_field(&self, scene: &mut Scene) {
        match self {
            Modes::None(executor) => executor.lock().unwrap().clear_field(scene),
            Modes::Kuznechik(executor) => executor.lock().unwrap().clear_field(scene),
            Modes::Vodolei(executor) => executor.lock().unwrap().clear_field(scene),
            Modes::Cherepaha(executor) => executor.lock().unwrap().clear_field(scene),
            Modes::Chertezhnik(executor) => executor.lock().unwrap().clear_field(scene),
            Modes::Robot(executor) => executor.lock().unwrap().clear_field(scene),
        }
    }

    fn draw_field(&mut self, scene: &mut Scene) {
        match self {
            Modes::None(executor) => executor.lock().unwrap().draw_field(scene),
            Modes::Kuznechik(executor) => executor.lock().unwrap().draw_field(scene),
            Modes::Vodolei(executor) => executor.lock().unwrap().draw_field(scene),
            Modes::Cherepaha(executor) => executor.lock().unwrap().draw_field(scene),
            Modes::Chertezhnik(executor) => executor.lock().unwrap().draw_field(scene),
            Modes::Robot(executor) => executor.lock().unwrap().draw_field(scene),
        }
    }

    fn base_color(&self) -> Color {
        match self {
            Modes::None(executor) => executor.lock().unwrap().base_color(),
            Modes::Kuznechik(executor) => executor.lock().unwrap().base_color(),
            Modes::Vodolei(executor) => executor.lock().unwrap().base_color(),
            Modes::Cherepaha(executor) => executor.lock().unwrap().base_color(),
            Modes::Chertezhnik(executor) => executor.lock().unwrap().base_color(),
            Modes::Robot(executor) => executor.lock().unwrap().base_color(),
        }
    }

    fn change_scale(&mut self, delta_scale: f64) {
        match self {
            Modes::None(executor) => executor.lock().unwrap().change_scale(delta_scale),
            Modes::Kuznechik(executor) => executor.lock().unwrap().change_scale(delta_scale),
            Modes::Vodolei(executor) => executor.lock().unwrap().change_scale(delta_scale),
            Modes::Cherepaha(executor) => executor.lock().unwrap().change_scale(delta_scale),
            Modes::Chertezhnik(executor) => executor.lock().unwrap().change_scale(delta_scale),
            Modes::Robot(executor) => executor.lock().unwrap().change_scale(delta_scale),
        };
    }

    fn get_scale(&self) -> f64 {
        match self {
            Modes::None(executor) => executor.lock().unwrap().get_scale(),
            Modes::Kuznechik(executor) => executor.lock().unwrap().get_scale(),
            Modes::Vodolei(executor) => executor.lock().unwrap().get_scale(),
            Modes::Cherepaha(executor) => executor.lock().unwrap().get_scale(),
            Modes::Chertezhnik(executor) => executor.lock().unwrap().get_scale(),
            Modes::Robot(executor) => executor.lock().unwrap().get_scale(),
        }
    }

    fn hovered(&mut self, pos: Pos2, pixels_per_point: f32) {
        match self {
            Modes::None(executor) => executor.lock().unwrap().hovered(pos, pixels_per_point),
            Modes::Kuznechik(executor) => executor.lock().unwrap().hovered(pos, pixels_per_point),
            Modes::Vodolei(executor) => executor.lock().unwrap().hovered(pos, pixels_per_point),
            Modes::Cherepaha(executor) => executor.lock().unwrap().hovered(pos, pixels_per_point),
            Modes::Chertezhnik(executor) => executor.lock().unwrap().hovered(pos, pixels_per_point),
            Modes::Robot(executor) => executor.lock().unwrap().hovered(pos, pixels_per_point),
        }
    }

    fn clicked(&mut self) {
        match self {
            Modes::None(executor) => executor.lock().unwrap().clicked(),
            Modes::Kuznechik(executor) => executor.lock().unwrap().clicked(),
            Modes::Vodolei(executor) => executor.lock().unwrap().clicked(),
            Modes::Cherepaha(executor) => executor.lock().unwrap().clicked(),
            Modes::Chertezhnik(executor) => executor.lock().unwrap().clicked(),
            Modes::Robot(executor) => executor.lock().unwrap().clicked(),
        }
    }

    fn drag_started(&mut self) {
        match self {
            Modes::None(executor) => executor.lock().unwrap().drag_started(),
            Modes::Kuznechik(executor) => executor.lock().unwrap().drag_started(),
            Modes::Vodolei(executor) => executor.lock().unwrap().drag_started(),
            Modes::Cherepaha(executor) => executor.lock().unwrap().drag_started(),
            Modes::Chertezhnik(executor) => executor.lock().unwrap().drag_started(),
            Modes::Robot(executor) => executor.lock().unwrap().drag_started(),
        }
    }

    fn drag(&mut self, drag_delta: Vec2, pixels_per_point: f32) {
        match self {
            Modes::None(executor) => executor.lock().unwrap().drag(drag_delta, pixels_per_point),
            Modes::Kuznechik(executor) => {
                executor.lock().unwrap().drag(drag_delta, pixels_per_point)
            }
            Modes::Vodolei(executor) => executor.lock().unwrap().drag(drag_delta, pixels_per_point),
            Modes::Cherepaha(executor) => {
                executor.lock().unwrap().drag(drag_delta, pixels_per_point)
            }
            Modes::Chertezhnik(executor) => {
                executor.lock().unwrap().drag(drag_delta, pixels_per_point)
            }
            Modes::Robot(executor) => executor.lock().unwrap().drag(drag_delta, pixels_per_point),
        }
    }

    fn drag_stop(&mut self) {
        match self {
            Modes::None(executor) => executor.lock().unwrap().drag_stop(),
            Modes::Kuznechik(executor) => executor.lock().unwrap().drag_stop(),
            Modes::Vodolei(executor) => executor.lock().unwrap().drag_stop(),
            Modes::Cherepaha(executor) => executor.lock().unwrap().drag_stop(),
            Modes::Chertezhnik(executor) => executor.lock().unwrap().drag_stop(),
            Modes::Robot(executor) => executor.lock().unwrap().drag_stop(),
        }
    }

    fn update_transform(&mut self, width: f64, height: f64) {
        match self {
            Modes::None(executor) => executor.lock().unwrap().update_transform(width, height),
            Modes::Kuznechik(executor) => executor.lock().unwrap().update_transform(width, height),
            Modes::Vodolei(executor) => executor.lock().unwrap().update_transform(width, height),
            Modes::Cherepaha(executor) => executor.lock().unwrap().update_transform(width, height),
            Modes::Chertezhnik(executor) => {
                executor.lock().unwrap().update_transform(width, height)
            }
            Modes::Robot(executor) => executor.lock().unwrap().update_transform(width, height),
        }
    }
}

impl PartialEq for Modes {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

impl fmt::Display for Modes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Modes::None(_) => write!(f, "Не выбрано"),
            Modes::Kuznechik(_) => write!(f, "Кузнечик"),
            Modes::Vodolei(_) => write!(f, "Водолей"),
            Modes::Robot(_) => write!(f, "Робот"),
            Modes::Chertezhnik(_) => write!(f, "Чертежник"),
            Modes::Cherepaha(_) => write!(f, "Черепаха"),
        }
    }
}

// pub enum VisualMode {
//     Dark,
//     Light,
// }

pub struct ModesStored {
    pub none: Modes,
    pub kuznechik: Modes,
    pub vodolei: Modes,
    pub robot: Modes,
    pub chertezhnik: Modes,
    pub cherepaha: Modes,
}

pub struct KumirState {
    pub scene: Arc<Mutex<Scene>>,
    // pub width: f64,
    // pub height: f64,
    pub selected_mode: Modes,
    pub modes: ModesStored,
    // pub visual_mode: VisualMode,
    pub min_point: Pos2,
    pub kill_flag: Arc<AtomicBool>,
}

impl KumirState {
    pub fn new(scene: Arc<Mutex<Scene>>, width: f64, height: f64) -> KumirState {
        let none = Modes::None(Arc::new(Mutex::new(NoneSelected::new())));
        let kuznechik = Modes::Kuznechik(Arc::new(Mutex::new(NoneSelected::new())));
        let vodolei = Modes::Vodolei(Arc::new(Mutex::new(NoneSelected::new())));
        let robot = Modes::Robot(Arc::new(Mutex::new(Robot::new(
            9,
            9,
            100.0,
            width / 2.0,
            height / 2.0,
        ))));
        let chertezhnik = Modes::Chertezhnik(Arc::new(Mutex::new(NoneSelected::new())));
        let cherepaha = Modes::Cherepaha(Arc::new(Mutex::new(NoneSelected::new())));
        KumirState {
            scene: scene,
            // width: width,
            // height: height,
            selected_mode: none.clone(),
            modes: ModesStored {
                none: none,
                kuznechik: kuznechik,
                vodolei: vodolei,
                robot: robot,
                chertezhnik: chertezhnik,
                cherepaha: cherepaha,
            },
            // visual_mode: VisualMode::Dark,
            min_point: Pos2::new(10.0, 85.0),
            kill_flag: Default::default(),
        }
    }

    pub fn add_shapes_to_scene(&mut self) {
        self.selected_mode
            .draw_field(&mut self.scene.lock().unwrap());
    }

    pub fn update_transform(&mut self, width: f64, height: f64) {
        self.modes.cherepaha.update_transform(width, height);
        self.modes.chertezhnik.update_transform(width, height);
        self.modes.kuznechik.update_transform(width, height);
        self.modes.none.update_transform(width, height);
        self.modes.robot.update_transform(width, height);
        self.modes.vodolei.update_transform(width, height);
    }

    pub fn update_min_point(&mut self, pos: Pos2) {
        if self.min_point != pos {
            self.min_point = pos;
        }
    }

    pub fn change_scale(&mut self, scale_delta: f64) {
        self.selected_mode.change_scale(scale_delta);
    }

    // pub fn get_scale(&self) -> f64 {
    //     self.selected_mode.get_scale()
    // }

    pub fn base_color(&self) -> Color {
        self.selected_mode.base_color()
    }

    pub fn hover(&mut self, pos: Option<Pos2>, pixels_per_point: f32) {
        if pos == None {
            return;
        }

        self.selected_mode
            .hovered((pos.unwrap() - self.min_point).to_pos2(), pixels_per_point);
    }

    pub fn click(&mut self) {
        self.selected_mode.clicked();
    }

    pub fn drag_start(&mut self) {
        self.selected_mode.drag_started();
    }

    pub fn drag(&mut self, drag_delta: Vec2, pixels_per_point: f32) {
        self.selected_mode.drag(drag_delta, pixels_per_point);
    }

    pub fn drag_stop(&mut self) {
        self.selected_mode.drag_stop();
    }
}

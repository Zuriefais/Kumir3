use std::sync::{Arc, Mutex};

use crate::kumir_state::{EditingStates, KumirState, Modes, ModesStored};
use crate::widgets::panes::{
    IDEWindowOptions, Pane, TreeBehavior, VelloWindowOptions, create_tree,
};
use crate::widgets::robot_gui::RobotWidget;
use egui::Vec2;
use egui::{Context, TextureId, load::SizedTexture};
use egui::{Response, Sense, Ui, Widget};

use log::info;

pub struct KumirGui {
    egui_context: Context,
    kumir_state: KumirState,
    tree: egui_tiles::Tree<Pane>,
}

impl KumirGui {
    pub fn new(
        context: &Context,
        kumir_state: KumirState,
        vello_options: Arc<Mutex<VelloWindowOptions>>,
    ) -> Self {
        Self {
            egui_context: context.clone(),
            kumir_state: kumir_state,
            tree: create_tree(vello_options.clone()),
        }
    }

    pub fn render_gui(&mut self) {
        egui::TopBottomPanel::top("tools").show(&self.egui_context, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .add(egui::Button::new("Запустить").frame(false))
                    .clicked()
                {
                    self.kumir_state.run();
                    info!("Something should run");
                }

                if ui
                    .add(egui::Button::new("Остановить").frame(false))
                    .clicked()
                {
                    info!("Something should stop");
                }

                egui::ComboBox::from_id_salt("mode")
                    .selected_text(self.kumir_state.selected_mode.to_string())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.kumir_state.selected_mode,
                            Modes::None,
                            "Не выбрано",
                        );
                        ui.selectable_value(
                            &mut self.kumir_state.selected_mode,
                            Modes::Kuznechik,
                            "Кузнечик",
                        );
                        ui.selectable_value(
                            &mut self.kumir_state.selected_mode,
                            Modes::Vodolei,
                            "Водолей",
                        );
                        ui.selectable_value(
                            &mut self.kumir_state.selected_mode,
                            Modes::Robot,
                            "Робот",
                        );
                        ui.selectable_value(
                            &mut self.kumir_state.selected_mode,
                            Modes::Cherepaha,
                            "Черепаха",
                        );
                        ui.selectable_value(
                            &mut self.kumir_state.selected_mode,
                            Modes::Chertezhnik,
                            "Чертежник",
                        );
                    });
            })
        });

        // egui::TopBottomPanel::bottom("terminal")
        //     .resizable(true)
        //     // .default_height(200.0)
        //     // .max_height(f32::INFINITY)
        //     .show(&self.egui_context, |ui| {
        //         ui.label("Тут может быть вывод программы");
        //     });

        egui::CentralPanel::default().show(&self.egui_context, |ui| {
            let mut behavior = TreeBehavior {
                kumir_state: &mut self.kumir_state,
            };
            self.tree.ui(&mut behavior, ui);
        });

        egui::Window::new("Изменить поле")
            .resizable([true, false])
            .default_pos([15.0, 00.0])
            .show(&self.egui_context, |ui| {
                match self.kumir_state.selected_mode {
                    Modes::Robot => ui.add(RobotWidget {
                        kumir_state: &mut self.kumir_state,
                    }),
                    _ => ui.label("None"),
                }
            });
    }

    pub fn add_shapes_to_scene(&mut self) {
        self.kumir_state.add_shapes_to_scene();
    }
}

use std::sync::{Arc, Mutex};

use crate::kumir_state::{KumirState, Modes, ModesStored};
use crate::widgets::panes::{
    IDEWindowOptions, Pane, TreeBehavior, VelloWindowOptions, create_tree,
};
use crate::widgets::usage_diagnostics::UsageDiagnostics;
use egui::Vec2;
use egui::color_picker::Alpha;
use egui::{Context, TextureId, load::SizedTexture};
use egui::{Response, Sense, Ui, Widget};

use log::info;
use vello::peniko::color::{AlphaColor, Srgb};

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
                UsageDiagnostics {}.ui(ui);

                egui::ComboBox::from_id_salt("mode")
                    .selected_text(self.kumir_state.selected_mode.to_string())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.kumir_state.selected_mode,
                            self.kumir_state.modes.none.clone(),
                            "Не выбрано",
                        );
                        ui.selectable_value(
                            &mut self.kumir_state.selected_mode,
                            self.kumir_state.modes.kuznechik.clone(),
                            "Кузнечик",
                        );
                        ui.selectable_value(
                            &mut self.kumir_state.selected_mode,
                            self.kumir_state.modes.vodolei.clone(),
                            "Водолей",
                        );
                        ui.selectable_value(
                            &mut self.kumir_state.selected_mode,
                            self.kumir_state.modes.robot.clone(),
                            "Робот",
                        );
                        ui.selectable_value(
                            &mut self.kumir_state.selected_mode,
                            self.kumir_state.modes.cherepaha.clone(),
                            "Черепаха",
                        );
                        ui.selectable_value(
                            &mut self.kumir_state.selected_mode,
                            self.kumir_state.modes.chertezhnik.clone(),
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
    }

    pub fn add_shapes_to_scene(&mut self) {
        self.kumir_state.add_shapes_to_scene();
    }

    pub fn update_transform(&mut self, width: f64, height: f64) {
        self.kumir_state.update_transform(width, height);
    }

    pub fn base_color(&self) -> AlphaColor<Srgb> {
        self.kumir_state.base_color()
    }
}

use std::{fmt, sync::Arc, sync::RwLock};

use egui::{Context, TextureHandle, TextureId, epaint::image, load::SizedTexture};
use egui_extras::syntax_highlighting::{CodeTheme, highlight};
use log::info;

#[derive(Default)]
pub struct VelloWindowSize {
    pub height: u32,
    pub width: u32,
    pub changed: bool,
}

pub struct KumirGui {
    egui_context: Context,
    code: String,
    lang: String,
    selected_mode: Modes,
    vello_window_size: Arc<RwLock<VelloWindowSize>>,
}

#[derive(PartialEq, Eq, Clone)]
enum Modes {
    None,
    Kuznechik,
    Vodolei,
    Cherepaha,
    Chertezhnik,
    Robot,
}

struct Pane {
    nr: usize,
}

struct TreeBehavior {}

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

impl KumirGui {
    pub fn new(context: &Context, vello_window_size: Arc<RwLock<VelloWindowSize>>) -> Self {
        let gui = Self {
            egui_context: context.clone(),
            code: "fn main() {\n    println!(\"Hello, world!\");\n}".to_string(),
            lang: "Rust".to_string(),
            selected_mode: Modes::None,
            vello_window_size,
        };
        gui
    }

    pub fn render_gui(&mut self, vello_texture: TextureId) {
        egui::Window::new("Vello").show(&self.egui_context, |ui| {
            let available_size = ui.available_size();
            if let Ok(mut size) = self.vello_window_size.write() {
                if size.width != available_size.x as u32 || size.height != available_size.y as u32 {
                    size.width = available_size.x as u32;
                    size.height = available_size.y as u32;
                    size.changed = true;
                }
            }

            ui.image(SizedTexture {
                id: vello_texture,
                size: egui::Vec2::new(available_size.x, available_size.y),
            });
        });
        egui::SidePanel::left("IDE")
            .max_width(f32::INFINITY)
            .show(&self.egui_context, |ui| {
                ui.label("Самое современное IDE");
                ui.horizontal(|ui| {
                    ui.set_height(0.0);
                    ui.label("An example of syntax highlighting in a TextEdit.");
                });

                ui.horizontal(|ui| {
                    ui.label("Language:");
                    ui.text_edit_singleline(&mut self.lang);
                });

                // let mut theme = CodeTheme::from_memory(ui.ctx(), ui.style());
                let mut theme =
                    egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx(), ui.style());
                ui.collapsing("Theme", |ui| {
                    ui.group(|ui| {
                        theme.ui(ui);
                        theme.clone().store_in_memory(ui.ctx());
                    });
                });

                let mut layouter = |ui: &egui::Ui, buf: &str, wrap_width: f32| {
                    let lang = {
                        let lang = self.lang.to_lowercase();
                        let mut chars = lang.chars();
                        match chars.next() {
                            None => String::new(),
                            Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
                        }
                    };

                    let mut layout_job =
                        highlight(ui.ctx(), ui.style(), &theme, buf, lang.as_str());
                    layout_job.wrap.max_width = wrap_width;
                    ui.fonts(|f| f.layout_job(layout_job))
                };

                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.code)
                            .font(egui::TextStyle::Monospace)
                            .code_editor()
                            .desired_rows(10)
                            .lock_focus(true)
                            .desired_width(f32::INFINITY)
                            .layouter(&mut layouter),
                    );
                });
            });

        egui::TopBottomPanel::top("tools").show(&self.egui_context, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .add(egui::Button::new("Запустить").frame(false))
                    .clicked()
                {
                    info!("Something should run");
                }

                if ui
                    .add(egui::Button::new("Остановить").frame(false))
                    .clicked()
                {
                    info!("Something should stop");
                }

                egui::ComboBox::from_id_salt("mode")
                    .selected_text(self.selected_mode.to_string())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.selected_mode, Modes::None, "Не выбрано");
                        ui.selectable_value(&mut self.selected_mode, Modes::Kuznechik, "Кузнечик");
                        ui.selectable_value(&mut self.selected_mode, Modes::Vodolei, "Водолей");
                        ui.selectable_value(&mut self.selected_mode, Modes::Robot, "Робот");
                        ui.selectable_value(&mut self.selected_mode, Modes::Cherepaha, "Черепаха");
                        ui.selectable_value(
                            &mut self.selected_mode,
                            Modes::Chertezhnik,
                            "Чертежник",
                        );
                    });
            })
        });

        egui::TopBottomPanel::bottom("terminal")
            .resizable(true)
            // .default_height(200.0)
            // .max_height(f32::INFINITY)
            .show(&self.egui_context, |ui| {
                ui.label("Тут может быть вывод программы");
            });
    }
}

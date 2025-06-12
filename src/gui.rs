use std::fmt;

use egui::Context;
use egui_extras::syntax_highlighting::{CodeTheme, highlight};

pub struct KumirGui {
    egui_context: Context,
    code: String,
    lang: String,
    selected_mode: Modes,
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
    pub fn new(context: &Context) -> Self {
        let mut gui = Self {
            egui_context: context.clone(),
            code: "fn main() {\n    println!(\"Hello, world!\");\n}".to_string(),
            lang: "Rust".to_string(),
            selected_mode: Modes::None,
        };
        gui
    }

    pub fn render_gui(&mut self) {
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
                if ui.add(egui::Button::new("Запусить").frame(false)).clicked() {
                    println!("Something should run");
                }

                if ui
                    .add(egui::Button::new("Остановить").frame(false))
                    .clicked()
                {
                    println!("Something should stop");
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
    }
}

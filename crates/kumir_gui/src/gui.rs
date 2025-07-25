use std::{
    fmt,
    sync::{Arc, Mutex},
};

use egui::{Context, TextureId, load::SizedTexture};
use egui_extras::syntax_highlighting::highlight;

use log::info;

#[derive(Default)]
pub struct VelloWindowOptions {
    pub height: u32,
    pub width: u32,
    pub texture: TextureId,
    pub changed: bool,
}

pub struct IDEWindowOptions {
    code: String,
    lang: String,
}

pub struct KumirGui {
    egui_context: Context,

    selected_mode: Modes,
    tree: egui_tiles::Tree<Pane>,
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
    pub fn new(context: &Context, vello_options: Arc<Mutex<VelloWindowOptions>>) -> Self {
        let gui = Self {
            egui_context: context.clone(),

            selected_mode: Modes::None,

            tree: create_tree(vello_options.clone()),
        };
        gui
    }

    pub fn render_gui(&mut self) {
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

        egui::CentralPanel::default().show(&self.egui_context, |ui| {
            let mut behavior = TreeBehavior {};
            self.tree.ui(&mut behavior, ui);
        });
    }
}

enum Pane {
    Unknown(usize),
    Terminal,
    Tools,
    IDE(IDEWindowOptions),
    Vello(Arc<Mutex<VelloWindowOptions>>),
}

struct TreeBehavior {}

impl egui_tiles::Behavior<Pane> for TreeBehavior {
    fn tab_title_for_pane(&mut self, pane: &Pane) -> egui::WidgetText {
        match pane {
            Pane::Unknown(u) => format!("{}", u).into(),
            Pane::Terminal => format!("Terminal").into(),
            Pane::Tools => format!("Tools").into(),
            Pane::IDE(_) => format!("IDE").into(),
            Pane::Vello(_) => format!("Vello window").into(),
        }
    }
    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: egui_tiles::TileId,
        pane: &mut Pane,
    ) -> egui_tiles::UiResponse {
        // Create a draggable title bar
        let title_bar_response = ui
            .horizontal(|ui| {
                ui.set_min_height(24.0); // Standard title bar height
                ui.style_mut().visuals.extreme_bg_color = egui::Color32::DARK_GRAY; // Title bar color
                ui.style_mut().visuals.widgets.hovered.bg_fill = egui::Color32::GRAY; // Hover effect
                ui.style_mut().spacing.item_spacing = egui::vec2(0.0, 0.0); // No spacing
                ui.add_space(4.0); // Left padding
                ui.label(format!(
                    "Pane {}",
                    match pane {
                        Pane::Unknown(nr) => format!("{}", nr),
                        Pane::Terminal => format!("Terminal"),
                        Pane::Tools => format!("Tools"),
                        Pane::IDE(_) => format!("IDE"),
                        Pane::Vello(_) => format!("Vello"),
                    }
                )); // Title text
                ui.allocate_space(ui.available_size()); // Fill remaining space
                ui.interact(
                    ui.max_rect(),
                    ui.id().with("title_bar"),
                    egui::Sense::drag(),
                )
            })
            .inner;

        match pane {
            Pane::Unknown(nr) => {
                ui.label(format!("The contents of pane {}.", nr));
            }
            Pane::Terminal => todo!(),
            Pane::Tools => todo!(),
            Pane::IDE(options) => {
                ui.label("Самое современное IDE");
                ui.horizontal(|ui| {
                    ui.set_height(0.0);
                    ui.label("An example of syntax highlighting in a TextEdit.");
                });

                ui.horizontal(|ui| {
                    ui.label("Language:");
                    ui.text_edit_singleline(&mut options.lang);
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
                        let lang = options.lang.to_lowercase();
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
                        egui::TextEdit::multiline(&mut options.code)
                            .font(egui::TextStyle::Monospace)
                            .code_editor()
                            .desired_rows(10)
                            .lock_focus(true)
                            .desired_width(f32::INFINITY)
                            .layouter(&mut layouter),
                    );
                });
            }
            Pane::Vello(vello_options) => {
                let available_size = ui.available_size() * ui.ctx().pixels_per_point();
                if let Ok(mut vello_options) = vello_options.lock() {
                    if vello_options.width != available_size.x as u32
                        || vello_options.height != available_size.y as u32
                    {
                        vello_options.width = available_size.x as u32;
                        vello_options.height = available_size.y as u32;
                        vello_options.changed = true;
                    }
                    ui.image(SizedTexture {
                        id: vello_options.texture,
                        size: egui::Vec2::new(available_size.x, available_size.y),
                    });
                }
            }
        }
        if title_bar_response.drag_started() {
            egui_tiles::UiResponse::DragStarted
        } else {
            egui_tiles::UiResponse::None
        }
    }
}

fn create_tree(vello_options: Arc<Mutex<VelloWindowOptions>>) -> egui_tiles::Tree<Pane> {
    let mut next_view_nr = 0;
    let mut gen_pane = || {
        let pane = Pane::Unknown(next_view_nr);
        next_view_nr += 1;
        pane
    };

    let mut tiles = egui_tiles::Tiles::default();

    let mut tabs = vec![];

    tabs.push(tiles.insert_pane(Pane::Vello(vello_options)));
    tabs.push(tiles.insert_pane(gen_pane()));
    tabs.push(tiles.insert_pane(Pane::IDE(IDEWindowOptions {
        code: "fn main() {\n    println!(\"Hello, world!\");\n}".to_string(),
        lang: "Rust".to_string(),
    })));

    let root = tiles.insert_tab_tile(tabs);

    egui_tiles::Tree::new("my_tree", root, tiles)
}

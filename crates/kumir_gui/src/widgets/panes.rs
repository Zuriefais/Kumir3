use crate::kumir_state::{KumirState, Modes};
use crate::rustpy::run;
use crate::widgets::robot_gui::RobotWidget;
use egui::{Align2, Pos2, Sense, TextureId, Vec2, load::SizedTexture};
use egui_extras::syntax_highlighting::highlight;
use log::info;
use std::fmt;
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct VelloWindowOptions {
    pub height: u32,
    pub width: u32,
    pub texture: TextureId,
    pub changed: bool,
}

#[derive(PartialEq)]
pub enum Lang {
    Python,
    KumirLang,
}

impl fmt::Display for Lang {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Lang::Python => write!(f, "Python"),
            Lang::KumirLang => write!(f, "КуМир"),
        }
    }
}
pub struct IDEWindowOptions {
    code: String,
    lang: Lang,
}

pub enum Pane {
    Unknown(usize),
    Terminal,
    IDE(IDEWindowOptions),
    Vello(Arc<Mutex<VelloWindowOptions>>),
}

pub struct TreeBehavior<'a> {
    pub kumir_state: &'a mut KumirState,
}

impl egui_tiles::Behavior<Pane> for TreeBehavior<'_> {
    fn tab_title_for_pane(&mut self, pane: &Pane) -> egui::WidgetText {
        match pane {
            Pane::Unknown(u) => format!("{u}").into(),
            Pane::Terminal => "Terminal".to_string().into(),
            Pane::IDE(_) => "IDE".to_string().into(),
            Pane::Vello(_) => "Vello window".to_string().into(),
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
                        Pane::Unknown(nr) => format!("{nr}"),
                        Pane::Terminal => "Terminal".to_string(),
                        Pane::IDE(_) => "IDE".to_string(),
                        Pane::Vello(_) => "Vello".to_string(),
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
                ui.label(format!("The contents of pane {nr}."));
            }
            Pane::Terminal => todo!(),
            Pane::IDE(options) => {
                ui.label("Самое современное IDE");
                ui.horizontal(|ui| {
                    ui.set_height(0.0);
                    ui.label("An example of syntax highlighting in a TextEdit.");
                });
                ui.end_row();

                // ui.horizontal(|ui: &egui::Ui| {
                //     ui.label("Language:");
                //     // ui.text_edit_singleline(&mut options.lang);

                // });

                egui::ComboBox::from_label("<- language")
                    .selected_text(format!("{}", options.lang))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut options.lang, Lang::Python, "Python");
                        ui.selectable_value(&mut options.lang, Lang::KumirLang, "Кумир");
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

                ui.horizontal(|ui| {
                    if ui.add(egui::Button::new("Запустить")).clicked() {
                        run::run_code_from_string(
                            options.code.as_str(),
                            self.kumir_state.modes.clone(),
                        );
                        info!("Something should run");
                    }

                    if ui.add(egui::Button::new("Остановить")).clicked() {
                        info!("Something should stop");
                    }
                });
                let mut layouter = |ui: &egui::Ui, buf: &str, wrap_width: f32| {
                    let lang = format!("{}", options.lang);
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
                egui::Window::new("Параметры окна и поля")
                    .resizable([false, false])
                    .constrain_to(ui.available_rect_before_wrap())
                    .anchor(Align2::RIGHT_TOP, [-10.0, 10.0])
                    .show(&ui.ctx().clone(), |ui| {
                        match self.kumir_state.selected_mode {
                            Modes::Robot => ui.add(RobotWidget {
                                kumir_state: &mut self.kumir_state,
                            }),
                            _ => ui.label("Режим не выбран"),
                        }
                    });

                let available_size = ui.available_size() * ui.ctx().pixels_per_point();
                if let Ok(mut vello_options) = vello_options.lock() {
                    if vello_options.width != available_size.x as u32
                        || vello_options.height != available_size.y as u32
                    {
                        vello_options.width = available_size.x as u32;
                        vello_options.height = available_size.y as u32;
                        vello_options.changed = true;
                    }

                    let response = ui
                        .image(SizedTexture {
                            id: vello_options.texture,
                            size: egui::Vec2::new(available_size.x, available_size.y),
                        })
                        .interact(Sense::click());

                    self.kumir_state.update_min_point(response.rect.min);

                    if response.hovered() {
                        ui.input(|input: &'_ egui::InputState| {
                            let zoom_delta = input.zoom_delta();
                            self.kumir_state.change_scale(zoom_delta as f64 - 1.0);
                            self.kumir_state.hover(input.pointer.hover_pos());
                            // println!("{:?}", input.pointer.hover_pos());
                        });
                    }

                    if response.clicked() {
                        self.kumir_state.click();
                    }
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

pub fn create_tree(vello_options: Arc<Mutex<VelloWindowOptions>>) -> egui_tiles::Tree<Pane> {
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
        code: "print(\"Hello, world!\")\n".to_string(),
        lang: Lang::Python,
    })));

    let root = tiles.insert_tab_tile(tabs);

    egui_tiles::Tree::new("my_tree", root, tiles)
}

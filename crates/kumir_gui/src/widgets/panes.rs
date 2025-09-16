use crate::kumir_state::{KumirState, Modes};
use crate::runtime_requirements::GuiRuntimeRequirements;
use crate::widgets::{robot_gui::RobotWidget, terminal::Terminal};
use egui::Ui;
use egui::{Align2, Sense, TextureId, load::SizedTexture};
use egui_extras::syntax_highlighting::{CodeTheme, highlight};
use kumir_gui_docs::Docs;
use kumir_runtime::{Lang, Runtime};
use log::{error, info};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use wasm_thread as thread;

#[derive(Default)]
pub struct VelloWindowOptions {
    pub height: u32,
    pub width: u32,
    pub texture: TextureId,
    pub changed: bool,
}

pub struct IDEWindowOptions {
    code: String,
    lang: Lang,
    sleep_duration: u64,
}

pub enum Pane {
    Unknown(usize),
    #[allow(dead_code)]
    Terminal,
    IDE(IDEWindowOptions),
    Vello(Arc<Mutex<VelloWindowOptions>>),
    Docs,
}

pub struct TreeBehavior<'a> {
    pub kumir_state: &'a mut KumirState,
    pub docs: &'a mut Docs,
    ide_theme: CodeTheme,
}

impl<'a> TreeBehavior<'a> {
    pub fn new(kumir_state: &'a mut KumirState, docs: &'a mut Docs) -> Self {
        Self {
            kumir_state,
            ide_theme: CodeTheme::default(),
            docs,
        }
    }
}

impl egui_tiles::Behavior<Pane> for TreeBehavior<'_> {
    fn tab_title_for_pane(&mut self, pane: &Pane) -> egui::WidgetText {
        match pane {
            Pane::Unknown(u) => format!("{u}").into(),
            Pane::Terminal => format!("Terminal").into(),
            Pane::IDE(_) => "IDE".to_string().into(),
            Pane::Vello(_) => "Vello window".to_string().into(),
            Pane::Docs => format!("Docs").into(),
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
                ui.add_space(4.0);
                ui.label(self.tab_title_for_pane(pane));
                ui.allocate_space(ui.available_size());
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
            Pane::Terminal => {
                ui.add(Terminal {});
            }
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

                ui.horizontal(|ui: &mut Ui| {
                    ui.label("Язык программирования: ");
                    egui::ComboBox::from_id_salt("language")
                        .selected_text(format!("{}", options.lang))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut options.lang, Lang::Python, "Python");
                            ui.selectable_value(&mut options.lang, Lang::Kumir, "Кумир");
                        });
                });

                // let mut theme = CodeTheme::from_memory(ui.ctx(), ui.style());

                ui.horizontal(|ui| {
                    ui.label("Задержка действий исполнителя: ");
                    ui.add(egui::DragValue::new(&mut options.sleep_duration).speed(1));
                });

                ui.horizontal(|ui| {
                    if ui.add(egui::Button::new("Запустить")).clicked() {
                        let mode = self.kumir_state.selected_mode.clone();
                        let lang = options.lang.clone();
                        let code = options.code.clone();
                        let duration = options.sleep_duration.clone();
                        let kill_flag = self.kumir_state.kill_flag.clone();
                        let scene_is_dirty = self.kumir_state.scene_is_dirty.clone();
                        thread::spawn(move || {
                            info!("Starting runtime");

                            let mut target = kumir_runtime::Target::init(
                                Arc::new(GuiRuntimeRequirements {
                                    mode: mode,
                                    sleep_duration: Duration::from_millis(duration),
                                }),
                                lang,
                                code,
                                kill_flag,
                            )
                            .unwrap();
                            if let Err(err) = target.run() {
                                error!("{err}")
                            };
                            info!("Something should run");
                        });
                    }

                    if ui.add(egui::Button::new("Остановить")).clicked() {
                        self.kumir_state
                            .kill_flag
                            .store(true, std::sync::atomic::Ordering::Relaxed);
                        info!("Something should stop");
                    }
                });

                let mut layouter = |ui: &egui::Ui, buf: &str, wrap_width: f32| {
                    let lang = format!("{}", options.lang);
                    let mut layout_job =
                        highlight(ui.ctx(), ui.style(), &self.ide_theme, buf, lang.as_str());
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
                        ui.scope(|ui| {
                            match self.kumir_state.selected_mode {
                                Modes::Robot(_) => ui.add(RobotWidget {
                                    kumir_state: &mut self.kumir_state,
                                }),
                                _ => ui.label("Режим не выбран"),
                            };

                            self.ide_theme =
                                egui_extras::syntax_highlighting::CodeTheme::from_memory(
                                    ui.ctx(),
                                    ui.style(),
                                );
                            ui.collapsing("Theme", |ui| {
                                ui.group(|ui| {
                                    self.ide_theme.ui(ui);
                                    self.ide_theme.clone().store_in_memory(ui.ctx());
                                });
                            });
                        })
                    });

                let available_size = ui.available_size();
                if let Ok(mut vello_options) = vello_options.lock() {
                    let ppp = ui.ctx().pixels_per_point();
                    if vello_options.width != (available_size.x * ppp) as u32
                        || vello_options.height != (available_size.y * ppp) as u32
                    {
                        vello_options.width = (available_size.x * ppp) as u32;
                        vello_options.height = (available_size.y * ppp) as u32;
                        vello_options.changed = true;
                    }

                    let response = ui
                        .image(SizedTexture {
                            id: vello_options.texture,
                            size: egui::Vec2::new(available_size.x, available_size.y),
                        })
                        .interact(Sense::click_and_drag());

                    self.kumir_state.update_min_point(response.rect.min);

                    if response.hovered() {
                        ui.input(|input: &'_ egui::InputState| {
                            let zoom_delta = input.zoom_delta();
                            self.kumir_state.change_scale(zoom_delta as f64 - 1.0);
                            self.kumir_state
                                .hover(input.pointer.hover_pos(), ui.pixels_per_point());
                            // println!("{:?}", input.pointer.hover_pos());
                        });
                    }

                    if response.is_pointer_button_down_on() {
                        self.kumir_state.drag_start();
                    }

                    if response.clicked() {
                        self.kumir_state.click();
                    }

                    if response.dragged() {
                        self.kumir_state
                            .drag(response.drag_delta(), ui.pixels_per_point());
                    }

                    if response.drag_stopped() {
                        self.kumir_state.drag_stop();
                    }
                }
            }
            Pane::Docs => {
                self.docs.ui(ui);
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
        sleep_duration: 200,
    })));
    tabs.push(tiles.insert_pane(Pane::Terminal));
    tabs.push(tiles.insert_pane(Pane::Docs));

    let root = tiles.insert_tab_tile(tabs);

    egui_tiles::Tree::new("my_tree", root, tiles)
}

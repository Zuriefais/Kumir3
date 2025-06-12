use egui::Context;

pub struct KumirGui {
    egui_context: Context,
    code: String,
    lang: String,
}

impl KumirGui {
    pub fn new(context: &Context) -> Self {
        Self {
            egui_context: context.clone(),
            code: format!("Some rust code"),
            lang: format!("rust"),
        }
    }

    pub fn render_gui(&mut self) {
        egui::SidePanel::left("IDE").show(&self.egui_context, |ui| {
            ui.label("Самое современное IDE");
            ui.horizontal(|ui| {
                ui.set_height(0.0);
                ui.label("An example of syntax highlighting in a TextEdit.");
            });

            ui.horizontal(|ui| {
                ui.label("Language:");
                ui.text_edit_singleline(&mut self.lang);
            });
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.label("Syntax highlighting powered by ");
                ui.hyperlink_to("syntect", "https://github.com/trishume/syntect");
                ui.label(".");
            });

            let mut theme =
                egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx(), ui.style());
            ui.collapsing("Theme", |ui| {
                ui.group(|ui| {
                    theme.ui(ui);
                    theme.clone().store_in_memory(ui.ctx());
                });
            });

            let mut layouter = |ui: &egui::Ui, buf: &str, wrap_width: f32| {
                let mut layout_job = egui_extras::syntax_highlighting::highlight(
                    ui.ctx(),
                    ui.style(),
                    &theme,
                    buf,
                    "rust",
                );
                layout_job.wrap.max_width = wrap_width;
                ui.fonts(|f| f.layout_job(layout_job))
            };

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut self.code)
                        .font(egui::TextStyle::Monospace) // for cursor height
                        .code_editor()
                        .desired_rows(10)
                        .lock_focus(true)
                        .desired_width(f32::INFINITY)
                        .layouter(&mut layouter),
                );
            });
        });
    }
}

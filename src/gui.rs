use egui::Context;

pub struct KumirGui {
    egui_context: Context,
}

impl KumirGui {
    pub fn new(context: &Context) -> Self {
        Self {
            egui_context: context.clone(),
        }
    }

    pub fn render_gui(&mut self) {
        egui::SidePanel::left("IDE")
            .show(&self.egui_context, |ui| ui.label("Самое современное IDE"));
    }
}

use egui::{Response, Ui, Widget};

pub struct Terminal {}

impl Widget for Terminal {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.scope(|ui: &mut Ui| {
            ui.label("nothing to see here");
        })
        .response
    }
}

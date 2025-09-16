use egui::Ui;
use egui_commonmark::CommonMarkCache;

use kumir_gui_docs_macros::docs;

pub struct Docs {
    cache: CommonMarkCache,
    selected: String,
}
impl Docs {
    pub fn ui(&mut self, ui: &mut Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            docs!(
                ui,
                &mut self.cache,
                "userdocs/md_output",
                &mut self.selected
            )
        });
    }
}

impl Default for Docs {
    fn default() -> Self {
        Self {
            cache: Default::default(),
            selected: Default::default(),
        }
    }
}

use egui::{Ui, Widget};
use egui_commonmark::{CommonMarkCache, commonmark_str};

use kumir_gui_docs_macros::docs;
use web_time::Instant;

pub struct Docs {
    cache: CommonMarkCache,
}
impl Docs {
    pub fn ui(&mut self, ui: &mut Ui) {
        egui::ScrollArea::vertical()
            .show(ui, |ui| docs!(ui, &mut self.cache, "userdocs/md_output"));
    }
}

impl Default for Docs {
    fn default() -> Self {
        Self {
            cache: Default::default(),
        }
    }
}

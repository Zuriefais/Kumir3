use egui::{Ui, Widget};

use web_time::Instant;

pub struct UsageDiagnostics;

impl Widget for UsageDiagnostics {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        // Access stored data via Context
        let ctx = ui.ctx().clone();
        let mut state = ctx.memory_mut(|mem| {
            mem.data
                .get_temp::<DiagnosticsState>(egui::Id::new("usage_diagnostics"))
                .unwrap_or_default()
        });

        // Update diagnostics
        state.frame_count += 1;
        let now = Instant::now();
        let elapsed = now.duration_since(state.last_frame_time).as_secs_f32();
        if elapsed >= 1.0 {
            state.fps = state.frame_count as f32 / elapsed;
            state.frame_count = 0;
            state.last_frame_time = now;
        }

        // Store updated state
        ctx.memory_mut(|mem| {
            mem.data
                .insert_temp(egui::Id::new("usage_diagnostics"), state);
        });

        // Render widget
        egui::Window::new("Stats")
            .show(&ctx, |ui| {
                ui.label(format!("FPS: {:.2}", state.fps));
            })
            .map(|response| response.response)
            .unwrap_or_else(|| ui.label("No response"))
    }
}

#[derive(Clone, Copy)]
struct DiagnosticsState {
    frame_count: u32,
    last_frame_time: Instant,
    fps: f32,
}

impl Default for DiagnosticsState {
    fn default() -> Self {
        Self {
            frame_count: Default::default(),
            last_frame_time: Instant::now(),
            fps: Default::default(),
        }
    }
}

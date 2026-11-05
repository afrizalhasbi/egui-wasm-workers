use crate::engine_handle::Engine;
use crate::engine_handle::EngineImpl;
use std::time::Duration;

pub fn view(engine: &mut EngineImpl, ui: &mut egui::Ui) {
    ui.heading("1. Engine thread (wrapping i64 counter)");
    ui.label("Runs on a dedicated thread (native) / Web Worker + SharedArrayBuffer (wasm). UI never blocks.");

    if engine.running() {
        // Wake the UI periodically so the counter display stays live.
        ui.ctx().request_repaint_after(Duration::from_millis(50));
    }
    // Also wake when there is a new worker error to show.
    if engine.last_error().is_some() {
        ui.ctx().request_repaint();
    }

    ui.horizontal(|ui| {
        let label = if engine.running() {
            "Stop incrementing"
        } else {
            "Start incrementing"
        };
        if ui.button(label).clicked() {
            engine.toggle();
        }
        ui.monospace(format!("counter: {}", engine.counter()));
    });

    #[cfg(target_arch = "wasm32")]
    ui.label(engine.diagnostics());

    if let Some(err) = engine.last_error() {
        ui.colored_label(egui::Color32::RED, format!("worker error: {err}"));
    }
}

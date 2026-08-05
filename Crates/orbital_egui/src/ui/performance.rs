use egui::Ui;

use super::Panel;

/// Panel that displays frame timing and performance statistics.
///
/// Reads data from the egui context and (optionally) engine timing resources.
pub struct PerformancePanel;

impl Panel for PerformancePanel {
    fn ui(&mut self, ui: &mut Ui) {
        ui.heading("Frame Info");
        ui.separator();

        let viewport_info = ui.ctx().input(|i| {
            let screen_rect = i.raw.screen_rect.unwrap_or(egui::Rect::ZERO);
            let dt = i.predicted_dt;
            (screen_rect.width(), screen_rect.height(), dt)
        });

        ui.horizontal(|ui| {
            ui.label("Viewport:");
            ui.monospace(format!(
                "{:.0} x {:.0}",
                viewport_info.0, viewport_info.1
            ));
        });
        ui.horizontal(|ui| {
            ui.label("Predicted dt:");
            ui.monospace(format!("{:.2} ms", viewport_info.2 * 1000.0));
        });
        ui.horizontal(|ui| {
            ui.label("Approx FPS:");
            let fps = if viewport_info.2 > 0.0 {
                1.0 / viewport_info.2
            } else {
                0.0
            };
            ui.monospace(format!("{:.1}", fps));
        });

        ui.separator();
        ui.heading("Render Stats");
        ui.separator();
        ui.label("GPU timestamp queries available via");
        ui.label("Renderer timing data in the engine.");
    }
}

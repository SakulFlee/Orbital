use egui::Ui;

use super::Panel;

/// Panel that displays scene entities and allows inspection.
pub struct ScenePanel;

impl Panel for ScenePanel {
    fn ui(&mut self, ui: &mut Ui) {
        ui.heading("Entities");
        ui.separator();

        ui.label("Scene inspection will be available");
        ui.label("when ECS integration is complete.");
    }
}

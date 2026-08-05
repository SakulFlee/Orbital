use orbital_egui::egui;
use orbital_egui::egui::Ui;
use orbital_egui::ui::Panel;

pub(crate) struct UiDemo;

impl Panel for UiDemo {
    fn ui(&mut self, ui: &mut Ui) {
        egui::Window::new("Orbital egui").show(ui, |ui| {
            ui.heading("Demo UI");
        });
    }
}
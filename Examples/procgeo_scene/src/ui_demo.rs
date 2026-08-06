use winit::keyboard::KeyCode;
use orbital::app::Module;
use orbital::ecs::{System, World};
use orbital::wgpu::{Device, Queue};
use orbital_egui::{egui, EguiModule, EguiPanels};
use orbital_egui::egui::Ui;
use orbital_egui::ui::Panel;
use crate::ui_demo;

pub(crate) struct UiDemo;

impl Module for UiDemo {
    fn setup(&self, ecs: &mut World, device: &Device, queue: &Queue) -> Vec<Box<dyn System>> {
        if let Some(mut panels) = ecs.get_resource_mut::<EguiPanels>() {
            panels.0.push(Box::new(UiDemo));
        }

        Vec::new()
    }
}

impl Panel for UiDemo {
    fn ui(&mut self, ui: &mut Ui) {
        egui::Window::new("Orbital egui").show(ui, |ui| {
            ui.heading("Demo UI");
        });
    }
}
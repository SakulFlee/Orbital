use crate::floating_panel::FloatingPanel;
use iced_winit::core::{Element, Theme};
use iced_wgpu::Renderer;

#[derive(Debug, Clone)]
pub enum Message {
    None,
    ButtonPressed(String),
    ClosePanel,
}

pub struct IcedState {
    title: String,
    button_label: String,
    panel_visible: bool,
}

impl Default for IcedState {
    fn default() -> Self {
        Self {
            title: "Orbital UI".to_string(),
            button_label: "Click Me".to_string(),
            panel_visible: true,
        }
    }
}

impl IcedState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn with_button_label(mut self, label: impl Into<String>) -> Self {
        self.button_label = label.into();
        self
    }

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
    }

    pub fn set_button_label(&mut self, label: impl Into<String>) {
        self.button_label = label.into();
    }

    pub fn handle_message(&mut self, message: Message) {
        match message {
            Message::ButtonPressed(id) => {
                log::info!("Button pressed: {}", id);
            }
            Message::ClosePanel => {
                log::info!("Panel closed");
                self.panel_visible = false;
            }
            Message::None => {}
        }
    }

    pub fn view(&self) -> Element<'_, Message, Theme, Renderer> {
        use iced_widget::{button, column, container, text};

        if !self.panel_visible {
            return container(text(""))
                .width(iced_core::Length::Fill)
                .height(iced_core::Length::Fill)
                .into();
        }

        let title_bar = text(&self.title)
            .size(14)
            .color(iced_winit::core::Color::WHITE);

        let btn = button(text(&self.button_label))
            .on_press(Message::ButtonPressed(self.button_label.clone()));

        let content = column![btn].spacing(10).padding(10);

        FloatingPanel::new(title_bar, content)
            .initial_position(80.0, 80.0)
            .on_close(Message::ClosePanel)
            .into()
    }
}

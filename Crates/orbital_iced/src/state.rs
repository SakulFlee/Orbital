use iced_winit::core::{Element, Theme};
use iced_wgpu::Renderer;

#[derive(Debug, Clone)]
pub enum Message {
    None,
    ButtonPressed(String),
}

pub struct IcedState {
    title: String,
    button_label: String,
}

impl Default for IcedState {
    fn default() -> Self {
        Self {
            title: "Orbital UI".to_string(),
            button_label: "Click Me".to_string(),
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
            Message::None => {}
        }
    }

    pub fn view(&self) -> Element<'_, Message, Theme, Renderer> {
        use iced_widget::{button, column, container, text};

        let title_text = text(&self.title)
            .size(24)
            .color(iced_winit::core::Color::WHITE);

        let btn = button(text(&self.button_label))
            .on_press(Message::ButtonPressed(self.button_label.clone()));

        let content = column![title_text, btn].spacing(10).padding(20);

        container(content)
            .width(iced_core::Length::Fill)
            .height(iced_core::Length::Fill)
            .center_x(iced_core::Length::Fill)
            .center_y(iced_core::Length::Fill)
            .into()
    }
}

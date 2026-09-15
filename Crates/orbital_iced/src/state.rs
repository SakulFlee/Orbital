use crate::floating_panel::FloatingPanel;
use iced_winit::core::{Element, Theme};
use iced_wgpu::Renderer;
use orbital_ecs::World;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum Message {
    None,
    ButtonPressed(String),
    ClosePanel,
}

type ViewFn = Arc<dyn Fn(&World) -> Element<'static, Message, Theme, Renderer> + Send + Sync>;

#[derive(Clone)]
pub struct IcedState {
    title: String,
    button_label: String,
    position: (f32, f32),
    is_window: bool,
    visible: bool,
    view_fn: Option<ViewFn>,
}

impl Default for IcedState {
    fn default() -> Self {
        Self {
            title: "Orbital UI".to_string(),
            button_label: "Click Me".to_string(),
            position: (80.0, 80.0),
            is_window: true,
            visible: true,
            view_fn: None,
        }
    }
}

impl IcedState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn titled(title: impl Into<String>) -> Self {
        Self::new().with_title(title)
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn with_button_label(mut self, label: impl Into<String>) -> Self {
        self.button_label = label.into();
        self
    }

    pub fn with_position(mut self, x: f32, y: f32) -> Self {
        self.position = (x, y);
        self
    }

    pub fn with_hud(mut self) -> Self {
        self.is_window = false;
        self
    }

    pub fn with_view(
        mut self,
        f: impl Fn(&World) -> Element<'static, Message, Theme, Renderer> + Send + Sync + 'static,
    ) -> Self {
        self.view_fn = Some(Arc::new(f));
        self
    }

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
    }

    pub fn set_button_label(&mut self, label: impl Into<String>) {
        self.button_label = label.into();
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    pub fn handle_message(&mut self, message: Message) {
        match message {
            Message::ButtonPressed(id) => {
                log::info!("Button pressed: {}", id);
            }
            Message::ClosePanel => {
                log::info!("Panel closed");
                self.visible = false;
            }
            Message::None => {}
        }
    }

    pub fn view(&self, ecs: &World) -> Element<'static, Message, Theme, Renderer> {
        if !self.visible {
            use iced_widget::container;
            return container(iced_widget::text(""))
                .width(iced_core::Length::Fill)
                .height(iced_core::Length::Fill)
                .into();
        }

        let inner = if let Some(ref f) = self.view_fn {
            f(ecs)
        } else {
            self.default_view()
        };

        if self.is_window {
            let title_bar = iced_widget::text(self.title.clone())
                .size(14)
                .color(iced_winit::core::Color::WHITE);

            FloatingPanel::new(title_bar, inner)
                .initial_position(self.position.0, self.position.1)
                .on_close(Message::ClosePanel)
                .into()
        } else {
            inner
        }
    }

    fn default_view(&self) -> Element<'static, Message, Theme, Renderer> {
        use iced_widget::{button, column, text};

        let btn = button(text(self.button_label.clone()))
            .on_press(Message::ButtonPressed(self.button_label.clone()));

        column![btn].spacing(10).padding(10).into()
    }
}

/// ECS resource — insert this to declare iced UI panels.
///
/// The [`IcedBridgeModule`](crate::IcedBridgeModule) will detect this resource
/// and automatically create an [`IcedLayerRenderer`](crate::IcedLayerRenderer) for each panel.
pub struct IcedUiState(pub HashMap<String, IcedState>);

impl IcedUiState {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn push(&mut self, name: impl Into<String>, state: IcedState) {
        self.0.insert(name.into(), state);
    }
}

impl Default for IcedUiState {
    fn default() -> Self {
        Self::new()
    }
}

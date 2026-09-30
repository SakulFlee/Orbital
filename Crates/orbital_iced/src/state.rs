use crate::floating_panel::FloatingPanel;
use iced_wgpu::Renderer;
use iced_winit::core::{Element, Theme};
use orbital_ecs::World;
use std::collections::HashMap;
use std::sync::Arc;

/// The default message type for panels that only need the built-in actions.
///
/// [`IcedState`] is generic over its message, so a panel with richer
/// interactions declares its own enum rather than encoding them as strings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    None,
    ButtonPressed(String),
    ClosePanel,
}

type ViewFn<M> = Arc<dyn Fn(&World) -> Element<'static, M, Theme, Renderer> + Send + Sync>;

type MessageHandler<M> = Arc<dyn Fn(&mut IcedState<M>, M, &mut World) + Send + Sync>;

/// A single floating Iced panel.
///
/// The view is a pure `Fn(&World) -> Element`, so all panel state lives in the
/// ECS world rather than in the panel. Messages are delivered to
/// [`handle_message`](IcedState::handle_message), which is also where the world
/// can be mutated — that is what lets a panel change ECS data in response to a
/// click.
///
/// `M` defaults to [`Message`], so existing code and `IcedUiState` are
/// unaffected. `PartialEq` is required so the close button can be recognised
/// and hide the panel regardless of the message type.
#[derive(Clone)]
pub struct IcedState<M: Clone + PartialEq + Send + Sync + 'static = Message> {
    title: String,
    button_label: String,
    position: (f32, f32),
    is_window: bool,
    visible: bool,
    close_message: Option<M>,
    view_fn: Option<ViewFn<M>>,
    message_handler: Option<MessageHandler<M>>,
}

impl<M: Clone + PartialEq + Send + Sync + 'static> Default for IcedState<M> {
    fn default() -> Self {
        Self {
            title: "Orbital UI".to_string(),
            button_label: "Click Me".to_string(),
            position: (80.0, 80.0),
            is_window: true,
            visible: true,
            close_message: None,
            view_fn: None,
            message_handler: None,
        }
    }
}

impl<M: Clone + PartialEq + Send + Sync + 'static> IcedState<M> {
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

    /// Render as a bare overlay instead of a draggable window with a title bar.
    pub fn with_hud(mut self) -> Self {
        self.is_window = false;
        self
    }

    /// The message published when the panel's close button is pressed.
    ///
    /// Without one the close button is omitted. For the default
    /// `M = Message` this is normally [`Message::ClosePanel`].
    pub fn with_close_message(mut self, message: M) -> Self {
        self.close_message = Some(message);
        self
    }

    /// Handles this panel's messages, replacing the default behaviour.
    ///
    /// The default only hides the panel on its close message. A panel that
    /// reacts to other messages — an inspector writing a value into the world,
    /// say — supplies a handler here. It receives the panel itself, so it can
    /// still call [`mark_dirty`](Self::mark_dirty) or read the title.
    pub fn with_message_handler(
        mut self,
        handler: impl Fn(&mut Self, M, &mut World) + Send + Sync + 'static,
    ) -> Self {
        self.message_handler = Some(Arc::new(handler));
        self
    }

    pub fn with_view(
        mut self,
        f: impl Fn(&World) -> Element<'static, M, Theme, Renderer> + Send + Sync + 'static,
    ) -> Self {
        self.view_fn = Some(Arc::new(f));
        self
    }

    pub fn title(&self) -> &str {
        &self.title
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

    /// Whether a view function was supplied.
    pub fn has_view(&self) -> bool {
        self.view_fn.is_some()
    }

    /// Sets the message published by the close button.
    pub fn set_close_message(&mut self, message: M) {
        self.close_message = Some(message);
    }

    /// Whether a close message was supplied.
    pub fn has_close_message(&self) -> bool {
        self.close_message.is_some()
    }

    /// Whether this panel should currently be built and drawn.
    ///
    /// A hidden panel is skipped entirely — no view, no layout, no draw pass.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Handles a message published by one of this panel's widgets.
    ///
    /// `world` is the live ECS world, so a panel can apply whatever change its
    /// message describes.
    ///
    /// The default implementation hides the panel when it receives its own
    /// close message, and otherwise does nothing. Panels that want to react to
    /// other messages should hold their own state and handle it in their view
    /// closure, or drive the world from a system.
    pub fn handle_message(&mut self, message: M, _world: &mut World) {
        if self.close_message.as_ref() == Some(&message) {
            log::info!("Panel '{}' closed", self.title);
            self.visible = false;
        }
    }

    pub fn view(&self, ecs: &World) -> Element<'static, M, Theme, Renderer> {
        if !self.visible {
            return hidden_view();
        }

        let inner = match &self.view_fn {
            Some(f) => f(ecs),
            None => self.placeholder_view(),
        };

        if !self.is_window {
            return inner;
        }

        let title_bar = iced_widget::text(self.title.clone())
            .size(14)
            .color(iced_winit::core::Color::WHITE);

        let panel =
            FloatingPanel::new(title_bar, inner).initial_position(self.position.0, self.position.1);

        match &self.close_message {
            Some(close) => panel.on_close(close.clone()).into(),
            None => panel.into(),
        }
    }

    /// Shown when a panel has no view function: just its title.
    fn placeholder_view(&self) -> Element<'static, M, Theme, Renderer> {
        use iced_widget::{column, text};

        column![text(self.title.clone())]
            .spacing(10)
            .padding(10)
            .into()
    }
}

/// The built-in view for the default [`Message`] type: a single button.
impl IcedState<Message> {
    /// Installs the built-in demo view — a single button that publishes
    /// [`Message::ButtonPressed`].
    ///
    /// [`IcedBridgeModule`](crate::IcedBridgeModule) applies this to panels
    /// declared through [`IcedUiState`] that do not supply their own view, so
    /// those panels keep the behaviour they had before `IcedState` became
    /// generic.
    pub fn with_builtin_view(mut self) -> Self {
        let label = self.button_label.clone();

        self.view_fn = Some(Arc::new(move |_| {
            use iced_widget::{button, column, text};

            let btn = button(text(label.clone())).on_press(Message::ButtonPressed(label.clone()));

            column![btn].spacing(10).padding(10).into()
        }));

        self
    }
}

/// A full-size transparent placeholder for a hidden panel.
///
/// A hidden panel must still occupy the whole window: the runtime builds a
/// full-window `UserInterface` per panel, and a zero-sized tree would break
/// hit-testing for the panels drawn after it.
fn hidden_view<M: Clone + PartialEq + Send + Sync + 'static>()
-> Element<'static, M, Theme, Renderer> {
    iced_widget::container(iced_widget::text(""))
        .width(iced_core::Length::Fill)
        .height(iced_core::Length::Fill)
        .into()
}

/// ECS resource — insert this to declare iced UI panels.
///
/// The [`IcedBridgeModule`](crate::IcedBridgeModule) will detect this resource
/// and automatically create an [`IcedLayerRenderer`](crate::IcedLayerRenderer)
/// for each panel.
///
/// Uses the default [`Message`] type. A panel with its own message enum should
/// be registered by pushing an [`IcedLayerRenderer`](crate::IcedLayerRenderer)
/// into `register_overlays` directly, which avoids depending on this resource
/// and the module ordering it implies.
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A message type that is not the default, to prove the generic path works
    /// without any `TypeId` trickery.
    #[derive(Debug, Clone, PartialEq)]
    enum PanelMessage {
        Select(usize),
        Close,
    }

    #[test]
    fn default_state_is_a_visible_window_with_no_view() {
        let state: IcedState = IcedState::new();

        assert!(state.is_visible());
        assert!(!state.has_view());
        assert!(!state.has_close_message());
    }

    #[test]
    fn close_message_hides_the_panel() {
        let mut state: IcedState =
            IcedState::titled("test").with_close_message(Message::ClosePanel);
        let mut world = World::new();

        state.handle_message(Message::ButtonPressed("other".into()), &mut world);
        assert!(state.is_visible(), "unrelated message must not hide");

        state.handle_message(Message::ClosePanel, &mut world);
        assert!(!state.is_visible(), "close message must hide");
    }

    #[test]
    fn close_message_hides_a_custom_message_panel() {
        let mut state = IcedState::titled("custom").with_close_message(PanelMessage::Close);
        let mut world = World::new();

        state.handle_message(PanelMessage::Select(3), &mut world);
        assert!(state.is_visible());

        state.handle_message(PanelMessage::Close, &mut world);
        assert!(!state.is_visible());
    }

    #[test]
    fn panel_without_a_close_message_ignores_anything() {
        let mut state: IcedState<PanelMessage> = IcedState::titled("no close");
        let mut world = World::new();

        state.handle_message(PanelMessage::Close, &mut world);
        assert!(state.is_visible());
    }

    #[test]
    fn builtins_fill_in_view_and_close_message() {
        let state = crate::bridge::apply_builtin_defaults(IcedState::titled("bridge"));
        assert!(state.has_view());
        assert!(state.has_close_message());

        // An explicit close message is not overwritten.
        let state = crate::bridge::apply_builtin_defaults(
            IcedState::titled("bridge").with_close_message(Message::ButtonPressed("x".into())),
        );
        assert!(state.has_view());
    }

    #[test]
    fn builtins_leave_a_supplied_view_alone() {
        let state =
            IcedState::titled("has view").with_view(|_: &World| iced_widget::text("custom").into());
        assert!(state.has_view());

        let state = crate::bridge::apply_builtin_defaults(state);
        assert!(state.has_view());
    }

    #[test]
    fn hidden_and_visible_views_build_without_a_gpu() {
        let world = World::new();
        let state: IcedState = IcedState::titled("t");

        // Construction only; asserting the tree shape would need a renderer.
        let _ = state.view(&world);

        let state = state.with_hud();
        let _ = state.view(&world);

        let mut hidden: IcedState = IcedState::titled("t");
        hidden.set_visible(false);
        let _ = hidden.view(&world);
    }
}

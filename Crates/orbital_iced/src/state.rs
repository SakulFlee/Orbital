use crate::floating_panel::FloatingPanel;
use iced_wgpu::Renderer;
use iced_winit::core::{Element, Length, Theme};
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

type VisibilityFn = Arc<dyn Fn(&World) -> bool + Send + Sync>;

/// Default maximum time between widget-tree rebuilds.
///
/// Four updates per second is enough for panels showing live game state while
/// keeping the cost of re-running the view, widget diff and layout low. Panels
/// that need smoother output can opt into [`SMOOTH_REFRESH_INTERVAL`], and
/// interactions are rebuilt immediately regardless of this interval.
pub const DEFAULT_REFRESH_INTERVAL: std::time::Duration = std::time::Duration::from_millis(250);

/// Maximum time between rebuilds for a panel that needs smooth output.
///
/// About 30 updates per second: visibly smoother than the lazy default while
/// still rebuilding less often than a full-frame refresh, which matters when
/// the view closure walks a large ECS world.
pub const SMOOTH_REFRESH_INTERVAL: std::time::Duration = std::time::Duration::from_millis(33);

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
    /// Extra condition, beyond [`visible`](Self::is_visible), deciding whether
    /// the panel is drawn. Lets panel state that lives in the ECS world (the
    /// debug overlay, for instance) hide the panel without the renderer having
    /// to keep a copy of that state in sync.
    visibility: Option<VisibilityFn>,
    /// Declared width/height of the window. `Shrink` (the default) sizes it to
    /// the content.
    width: Length,
    height: Length,
    /// Whether the window can be resized with its bottom-right grip.
    resizable: bool,
    /// Set when the panel's content needs rebuilding. Cleared by the renderer
    /// after each rebuild.
    dirty: bool,
    /// How long the renderer may reuse the previous widget tree before
    /// rebuilding it anyway. Defaults to [`DEFAULT_REFRESH_INTERVAL`]; zero
    /// means rebuild every frame.
    refresh_interval: std::time::Duration,
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
            visibility: None,
            width: Length::Shrink,
            height: Length::Shrink,
            resizable: true,
            dirty: true,
            refresh_interval: DEFAULT_REFRESH_INTERVAL,
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

    /// Hides the panel when `predicate` returns `false`.
    ///
    /// Unlike [`set_visible`](Self::set_visible), the condition is evaluated
    /// against the live ECS world each time the view is built, so state that
    /// lives in a resource — the debug overlay's on/off flag, for example — can
    /// hide the whole panel, title bar included, without the renderer holding a
    /// stale copy of it.
    pub fn with_visibility(
        mut self,
        predicate: impl Fn(&World) -> bool + Send + Sync + 'static,
    ) -> Self {
        self.visibility = Some(Arc::new(predicate));
        self
    }

    /// Sets the panel's declared width and height.
    ///
    /// `Shrink` sizes the panel to its content; a fixed length caps it (and a
    /// `Length::Fill` body fills it). The user can still resize the window.
    pub fn with_size(mut self, width: impl Into<Length>, height: impl Into<Length>) -> Self {
        self.width = width.into();
        self.height = height.into();
        self
    }

    /// Whether the bottom-right grip resizes this window. On by default.
    pub fn with_resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
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

    /// Whether this panel should currently be built and drawn, taking the
    /// world-driven [`visibility`](Self::with_visibility) predicate into
    /// account.
    ///
    /// Checked before building the view as well as by the renderer, so a panel
    /// hidden by the predicate does no update, layout or draw work at all.
    pub fn is_visible_in(&self, ecs: &World) -> bool {
        self.visible
            && self
                .visibility
                .as_ref()
                .is_none_or(|visibility| visibility(ecs))
    }

    /// Rebuild the widget tree at most this often.
    ///
    /// A panel whose content changes every frame does not need its view rebuilt
    /// every frame, and the rebuild is the expensive part: it re-runs the
    /// widget-tree diff and layout, and calls the view closure. With this set,
    /// the tree is still rebuilt immediately whenever
    /// [`mark_dirty`](Self::mark_dirty) has been called, and otherwise at most
    /// once per interval.
    ///
    /// Note that input is *not* throttled: events are still routed to the
    /// retained widget tree every frame, so a click is always hit-tested
    /// against a live layout. Only the redraw of the tree is rate-limited, and
    /// the consequence is that externally-driven changes — a component value
    /// moving, say — can appear up to one interval late.
    ///
    /// The default is [`DEFAULT_REFRESH_INTERVAL`] (250 ms, or 4 Hz). Use
    /// [`Duration::ZERO`](std::time::Duration::ZERO) to rebuild every frame.
    pub fn with_refresh_interval(mut self, interval: std::time::Duration) -> Self {
        self.refresh_interval = interval;
        self
    }

    /// Requests a rebuild on the next frame, bypassing
    /// [`refresh_interval`](Self::with_refresh_interval).
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Whether the widget tree is due for a rebuild.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Records that the tree was just rebuilt.
    pub fn mark_built(&mut self) {
        self.dirty = false;
    }

    /// How long the renderer may reuse the previous tree.
    pub fn refresh_interval(&self) -> std::time::Duration {
        self.refresh_interval
    }

    /// Handles a message published by one of this panel's widgets.
    ///
    /// `world` is the live ECS world, so a panel can apply whatever change its
    /// message describes.
    ///
    /// A panel that installed a [`with_message_handler`](Self::with_message_handler)
    /// owns all of its message handling: the handler is called with the panel
    /// and the world, and the default behaviour below is skipped. Otherwise the
    /// default hides the panel when it receives its own close message, and does
    /// nothing for any other message.
    pub fn handle_message(&mut self, message: M, world: &mut World) {
        // A message means something changed, so the tree is stale.
        self.dirty = true;

        // Cloning the `Arc` ends the borrow of `self` so the handler can take
        // `&mut self`. A panel with a handler is responsible for its own close
        // behaviour (it can still call `set_visible`), which is what lets a
        // debug panel route messages into the ECS world.
        if let Some(handler) = self.message_handler.clone() {
            handler(self, message, world);
            return;
        }

        if self.close_message.as_ref() == Some(&message) {
            log::info!("Panel '{}' closed", self.title);
            self.visible = false;
        }
    }

    pub fn view(&self, ecs: &World) -> Element<'static, M, Theme, Renderer> {
        // Checked before the view is built, so a hidden panel draws no title
        // bar either — that is what used to leave bare titles on screen while
        // the overlay was off.
        if !self.is_visible_in(ecs) {
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

        let panel = FloatingPanel::new(title_bar, inner)
            .width(self.width)
            .height(self.height)
            .resizable(self.resizable)
            .initial_position(self.position.0, self.position.1);

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
    fn a_new_panel_is_dirty_so_it_builds_on_the_first_frame() {
        let state: IcedState = IcedState::new();
        assert!(state.is_dirty());
    }

    #[test]
    fn marking_built_clears_dirty_and_mark_dirty_sets_it_again() {
        let mut state: IcedState = IcedState::new();

        state.mark_built();
        assert!(!state.is_dirty());

        state.mark_dirty();
        assert!(state.is_dirty());
    }

    #[test]
    fn handling_a_message_marks_the_panel_dirty() {
        let mut state: IcedState = IcedState::titled("t")
            .with_close_message(Message::ClosePanel)
            .with_refresh_interval(std::time::Duration::from_secs(10));
        let mut world = World::new();

        // Even an unrelated message, and even with a long refresh interval:
        // a message means something changed, so the tree must be rebuilt.
        state.mark_built();
        state.handle_message(Message::ButtonPressed("x".into()), &mut world);
        assert!(state.is_dirty());
    }

    #[test]
    fn the_default_refresh_interval_is_four_hz() {
        assert_eq!(
            IcedState::<Message>::new().refresh_interval(),
            DEFAULT_REFRESH_INTERVAL
        );
        assert_eq!(DEFAULT_REFRESH_INTERVAL.as_millis(), 250);
    }

    #[test]
    fn the_smooth_refresh_interval_is_thirty_hz() {
        assert_eq!(SMOOTH_REFRESH_INTERVAL.as_millis(), 33);
        assert!(
            SMOOTH_REFRESH_INTERVAL < DEFAULT_REFRESH_INTERVAL,
            "smooth mode must rebuild more often than the lazy default"
        );
    }

    #[test]
    fn zero_refresh_interval_opts_into_every_frame() {
        assert_eq!(
            IcedState::<Message>::new()
                .with_refresh_interval(std::time::Duration::ZERO)
                .refresh_interval(),
            std::time::Duration::ZERO,
            "zero means rebuild every frame"
        );
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

    #[test]
    fn visibility_predicate_can_hide_a_window() {
        use std::sync::atomic::{AtomicBool, Ordering};

        let visible = Arc::new(AtomicBool::new(false));
        let flag = visible.clone();

        let state: IcedState =
            IcedState::titled("t").with_visibility(move |_: &World| flag.load(Ordering::Relaxed));

        let world = World::new();

        assert!(!state.is_visible_in(&world));

        // The view must build in both cases without panicking; the predicate is
        // what decides whether a `FloatingPanel` is constructed at all.
        let _ = state.view(&world);
        visible.store(true, Ordering::Relaxed);
        assert!(state.is_visible_in(&world));
        let _ = state.view(&world);
    }

    #[test]
    fn the_static_visible_flag_overrides_the_visibility_predicate() {
        let mut state: IcedState = IcedState::titled("t").with_visibility(|_: &World| true);
        state.set_visible(false);

        assert!(!state.is_visible_in(&World::new()));
    }

    #[test]
    fn declared_size_and_resizability_are_stored() {
        let state: IcedState = IcedState::titled("t")
            .with_size(420.0, Length::Shrink)
            .with_resizable(false);

        assert_eq!(state.width, Length::Fixed(420.0));
        assert_eq!(state.height, Length::Shrink);
        assert!(!state.resizable);

        let state: IcedState = IcedState::titled("t");
        assert_eq!(state.width, Length::Shrink, "defaults to content sized");
        assert!(state.resizable, "windows are resizable by default");
    }
}

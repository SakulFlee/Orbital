use iced_core::alignment;
use iced_core::border::{self, Border};
use iced_core::event::Event;
use iced_core::layout::{self, Layout, Node};
use iced_core::mouse;
use iced_core::overlay;
use iced_core::renderer::{self, Quad};
use iced_core::shell::Shell;
use iced_core::text;
use iced_core::touch;
use iced_core::widget::{self, Tree};
use iced_core::{Color, Element, Font, Length, Pixels, Point, Rectangle, Size, Vector};

const TITLE_BAR_HEIGHT: f32 = 28.0;
const DRAG_DEADBAND: f32 = 3.0;
const RESIZE_GRIP: f32 = 16.0;
/// Keeps the grip clear of the content's scrollbar, which overlays the right
/// edge and would otherwise swallow the dots visually.
const GRIP_RIGHT_INSET: f32 = 12.0;

fn grip_bounds(bounds: Rectangle) -> Rectangle {
    Rectangle {
        x: bounds.x + (bounds.width - RESIZE_GRIP - GRIP_RIGHT_INSET).max(0.0),
        y: bounds.y + (bounds.height - RESIZE_GRIP).max(0.0),
        width: RESIZE_GRIP.min(bounds.width),
        height: RESIZE_GRIP.min(bounds.height),
    }
}
const MIN_PANEL_WIDTH: f32 = 200.0;
const MIN_CONTENT_HEIGHT: f32 = 60.0;

#[derive(Debug, Clone)]
pub struct State {
    position: Point,
    is_dragging: bool,
    drag_origin: Point,
    drag_offset: Vector,
    /// Content size chosen by dragging the resize grip. `None` until the panel
    /// is resized, so the declared size keeps working as the default.
    size: Option<Size>,
    is_resizing: bool,
    resize_origin: Point,
    resize_start: Size,
}

impl Default for State {
    fn default() -> Self {
        Self {
            position: Point::new(50.0, 50.0),
            is_dragging: false,
            drag_origin: Point::ORIGIN,
            drag_offset: Vector::new(0.0, 0.0),
            size: None,
            is_resizing: false,
            resize_origin: Point::ORIGIN,
            resize_start: Size::ZERO,
        }
    }
}

pub struct FloatingPanel<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    title: Element<'a, Message, Theme, Renderer>,
    content: Element<'a, Message, Theme, Renderer>,
    on_close: Option<Message>,
    width: Length,
    height: Length,
    max_height_fraction: Option<f32>,
    resizable: bool,
    initial_position: Option<Point>,
}

impl<'a, Message, Theme, Renderer> FloatingPanel<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: renderer::Renderer + 'a,
{
    pub fn new(
        title: impl Into<Element<'a, Message, Theme, Renderer>>,
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        Self {
            title: title.into(),
            content: content.into(),
            on_close: None,
            width: Length::Shrink,
            height: Length::Shrink,
            max_height_fraction: None,
            resizable: true,
            initial_position: None,
        }
    }

    pub fn on_close(mut self, message: Message) -> Self {
        self.on_close = Some(message);
        self
    }

    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// Caps the panel content at a fraction of the window height.
    ///
    /// The content is still laid out against the real window, but a `Length::Fill`
    /// body is bounded to `fraction * window.height` (or the space left below the
    /// panel, whichever is smaller) instead of stretching to the window edge.
    pub fn max_height_fraction(mut self, fraction: f32) -> Self {
        self.max_height_fraction = Some(fraction.max(0.0));
        self
    }

    /// Whether a bottom-right grip lets the user resize the panel.
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    pub fn initial_position(mut self, x: f32, y: f32) -> Self {
        self.initial_position = Some(Point::new(x, y));
        self
    }
}

impl<'a, Message, Theme, Renderer> widget::Widget<Message, Theme, Renderer>
    for FloatingPanel<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: 'a,
    Renderer: renderer::Renderer + text::Renderer + 'a,
{
    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<State>()
    }

    fn state(&self) -> widget::tree::State {
        let mut s = State::default();
        if let Some(pos) = self.initial_position {
            s.position = pos;
        }
        widget::tree::State::new(s)
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits) -> Node {
        // Tree::new() doesn't call diff(), so on first frame children may be empty
        if tree.children.len() < 2 {
            self.diff(tree);
        }

        let state = tree.state.downcast_mut::<State>();

        // A floating panel lives inside the window, so its children must be
        // constrained by the space actually left below and to the right of it.
        // Laying them out with `Limits::NONE` makes every `Length::Fill` child
        // resolve to infinity; the panel then gets an infinite node and its
        // background/border/title-bar quads are degenerate, so nothing but the
        // finite title text is drawn.
        let window = limits.max();
        let position = state.position;

        let available_width = (window.width - position.x).max(0.0);
        let available_height = (window.height - position.y - TITLE_BAR_HEIGHT).max(0.0);

        let (content_width, content_height) = match state.size {
            // A size chosen by dragging the grip wins over the declared one, but
            // is still clamped to the space actually available in the window.
            Some(size) => (
                size.width.clamp(0.0, available_width),
                size.height.clamp(0.0, available_height),
            ),
            None => {
                let content_width = match self.width {
                    Length::Fixed(width) => width.min(available_width),
                    _ => available_width,
                };

                let mut content_height = match self.height {
                    Length::Fixed(height) => (height - TITLE_BAR_HEIGHT).max(0.0),
                    _ => available_height,
                };

                if let Some(fraction) = self.max_height_fraction {
                    content_height = content_height.min((window.height * fraction).max(0.0));
                }

                (content_width, content_height)
            }
        };

        let title_limits =
            layout::Limits::new(Size::ZERO, Size::new(available_width, TITLE_BAR_HEIGHT));
        let content_limits =
            layout::Limits::new(Size::ZERO, Size::new(content_width, content_height));

        let title_widget = self.title.as_widget_mut();
        let title_size = title_widget.layout(
            tree.children.get_mut(0).unwrap_or(&mut Tree::empty()),
            renderer,
            &title_limits,
        );

        let content_widget = self.content.as_widget_mut();
        let content_size = content_widget.layout(
            tree.children.get_mut(1).unwrap_or(&mut Tree::empty()),
            renderer,
            &content_limits,
        );

        let panel_width = title_size
            .size()
            .width
            .max(content_size.size().width)
            .max(MIN_PANEL_WIDTH)
            .min(available_width);
        let panel_height = TITLE_BAR_HEIGHT + content_size.size().height;

        Node::with_children(
            Size::new(panel_width, panel_height),
            vec![
                title_size,
                content_size.move_to(Point::new(0.0, TITLE_BAR_HEIGHT)),
            ],
        )
        .move_to(state.position)
    }

    fn diff(&mut self, tree: &mut Tree) {
        if let widget::tree::State::None = &tree.state {
            tree.tag = self.tag();
            tree.state = self.state();
        }
        // Use diff_children to preserve existing child trees across frames.
        // Tree::new() creates fresh state (e.g. is_pressed = false), which
        // destroys widget state on every frame. diff_children reconciles
        // old/new children, keeping state for unchanged widgets alive.
        tree.diff_children(&mut [self.title.as_widget_mut(), self.content.as_widget_mut()]);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();
        let bounds = layout.bounds();

        let title_bounds = Rectangle {
            x: bounds.x,
            y: bounds.y,
            width: bounds.width,
            height: TITLE_BAR_HEIGHT,
        };

        let close_bounds = Rectangle {
            x: bounds.x + bounds.width - TITLE_BAR_HEIGHT,
            y: bounds.y,
            width: TITLE_BAR_HEIGHT,
            height: TITLE_BAR_HEIGHT,
        };

        let grip_bounds = grip_bounds(bounds);

        // Pointer input: mouse *and* touch. On touch-only platforms (Android)
        // winit emits no synthetic mouse events, so without the touch arms a
        // panel could neither be dragged nor closed with a finger.
        let (pressed_at, moved_to, released) = match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                (cursor.position(), None, false)
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => (None, None, true),
            Event::Mouse(mouse::Event::CursorMoved { position }) => (None, Some(*position), false),
            Event::Touch(touch::Event::FingerPressed { position, .. }) => {
                (Some(*position), None, false)
            }
            Event::Touch(touch::Event::FingerMoved { position, .. }) => {
                (None, Some(*position), false)
            }
            Event::Touch(touch::Event::FingerLifted { .. } | touch::Event::FingerLost { .. }) => {
                (None, None, true)
            }
            _ => (None, None, false),
        };

        if let Some(cursor_pos) = pressed_at {
            if self.resizable && grip_bounds.contains(cursor_pos) {
                state.is_resizing = true;
                state.resize_origin = cursor_pos;
                state.resize_start = Size::new(bounds.width, bounds.height);
                shell.capture_event();
            } else if title_bounds.contains(cursor_pos) && !close_bounds.contains(cursor_pos) {
                state.is_dragging = true;
                state.drag_origin = cursor_pos;
                state.drag_offset = cursor_pos - state.position;
                shell.capture_event();
            } else if close_bounds.contains(cursor_pos)
                && let Some(msg) = self.on_close.clone()
            {
                shell.publish(msg);
                shell.capture_event();
            }
        }

        if released && state.is_dragging {
            state.is_dragging = false;
            shell.capture_event();
        }

        if released && state.is_resizing {
            state.is_resizing = false;
            shell.capture_event();
        }

        if let Some(position) = moved_to
            && state.is_dragging
        {
            let delta = position - state.drag_origin;
            let dist = (delta.x * delta.x + delta.y * delta.y).sqrt();
            let offset_dist = (state.drag_offset.x * state.drag_offset.x
                + state.drag_offset.y * state.drag_offset.y)
                .sqrt();
            if dist > DRAG_DEADBAND || offset_dist > 0.0 {
                state.position = position - state.drag_offset;
                shell.request_redraw();
                shell.capture_event();
            }
        }

        if let Some(position) = moved_to
            && state.is_resizing
        {
            let delta = position - state.resize_origin;
            let width = (state.resize_start.width + delta.x).max(MIN_PANEL_WIDTH);
            let height =
                (state.resize_start.height + delta.y).max(TITLE_BAR_HEIGHT + MIN_CONTENT_HEIGHT);

            state.size = Some(Size::new(width, height - TITLE_BAR_HEIGHT));
            shell.request_redraw();
            shell.capture_event();
        }

        // Forward events to title bar child
        let title_layout = layout.child(0);
        self.title.as_widget_mut().update(
            tree.children.get_mut(0).unwrap(),
            event,
            title_layout,
            cursor,
            _renderer,
            shell,
            _viewport,
        );

        // Forward events to content child, unless the resize grip owns this
        // gesture: the grip sits over the content's scrollbar, and forwarding
        // would start a scroll while the panel is being resized.
        if !state.is_resizing {
            let content_layout = layout.child(1);
            self.content.as_widget_mut().update(
                tree.children.get_mut(1).unwrap(),
                event,
                content_layout,
                cursor,
                _renderer,
                shell,
                _viewport,
            );
        }

        // Whole-panel capture: any touch press/move inside the panel bounds
        // marks the panel as owning that finger — even when no interactive
        // widget was hit (body background, labels, text). Without this, a
        // finger resting on the panel body leaks to the game's touch controls
        // (virtual joystick / drag-to-look). Runs *after* the children above
        // so buttons/sliders inside the content keep consuming events first;
        // this only fires when nothing else captured the event.
        //
        // Touch-only by design: desktop mouse *hover* (CursorMoved) must keep
        // its previous sibling-overlap semantics, and mouse clicks are
        // handled by the dedicated arms above.
        if matches!(event, iced_core::event::Event::Touch(_))
            && !shell.is_event_captured()
            && let Some(position) = pressed_at.or(moved_to)
            && bounds.contains(position)
        {
            shell.capture_event();
        }

        // A wheel over a panel belongs to the panel even when the cursor is
        // not over a scrollable (or the scrollable is already at its end).
        // Without this, scrolling past the end of a list would leak into the
        // game's camera zoom.
        if matches!(
            event,
            iced_core::event::Event::Mouse(mouse::Event::WheelScrolled { .. })
        ) && !shell.is_event_captured()
            && cursor.position().is_some_and(|p| bounds.contains(p))
        {
            shell.capture_event();
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let state = tree.state.downcast_ref::<State>();

        // Panel background
        renderer.fill_quad(
            Quad {
                bounds,
                border: border::rounded(6.0)
                    .width(1.0)
                    .color(Color::from_rgba(0.3, 0.3, 0.4, 0.5)),
                ..Default::default()
            },
            Color::from_rgba(0.08, 0.08, 0.14, 0.9),
        );

        // Title bar background
        let title_bar_bounds = Rectangle {
            x: bounds.x,
            y: bounds.y,
            width: bounds.width,
            height: TITLE_BAR_HEIGHT,
        };
        renderer.fill_quad(
            Quad {
                bounds: title_bar_bounds,
                border: Border {
                    radius: border::Radius {
                        top_left: 6.0,
                        top_right: 6.0,
                        bottom_left: 0.0,
                        bottom_right: 0.0,
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            Color::from_rgba(0.15, 0.15, 0.25, 1.0),
        );

        // Close button
        let close_bounds = Rectangle {
            x: bounds.x + bounds.width - TITLE_BAR_HEIGHT,
            y: bounds.y,
            width: TITLE_BAR_HEIGHT,
            height: TITLE_BAR_HEIGHT,
        };
        let close_hover =
            !state.is_dragging && cursor.position().is_some_and(|p| close_bounds.contains(p));
        let close_bg = if close_hover {
            Color::from_rgba(0.8, 0.2, 0.2, 0.8)
        } else {
            Color::from_rgba(0.5, 0.15, 0.15, 0.6)
        };
        renderer.fill_quad(
            Quad {
                bounds: close_bounds,
                border: Border {
                    radius: border::Radius {
                        top_left: 0.0,
                        top_right: 6.0,
                        bottom_left: 0.0,
                        bottom_right: 0.0,
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            close_bg,
        );

        // Close button "X" text
        renderer.fill_text(
            text::Text {
                content: "X".to_string(),
                font: Font::DEFAULT,
                size: Pixels(13.0),
                line_height: text::LineHeight::Absolute(Pixels(16.0)),
                bounds: close_bounds.size(),
                align_x: text::Alignment::Center,
                align_y: alignment::Vertical::Center,
                shaping: text::Shaping::Basic,
                wrapping: text::Wrapping::None,
                ellipsis: text::Ellipsis::default(),
                hint_factor: None,
            },
            close_bounds.center(),
            Color::WHITE,
            *viewport,
        );

        // Title text
        let title_layout = layout.child(0);
        self.title.as_widget().draw(
            tree.children.first().unwrap(),
            renderer,
            theme,
            style,
            title_layout,
            cursor,
            viewport,
        );

        // Content
        let content_layout = layout.child(1);
        self.content.as_widget().draw(
            tree.children.get(1).unwrap(),
            renderer,
            theme,
            style,
            content_layout,
            cursor,
            viewport,
        );

        // Resize grip: a chip with three diagonal dots, inset from the right
        // edge so the content's scrollbar does not sit on top of it. Drawn last
        // so neither the content nor the scrollbar can cover it.
        if self.resizable {
            let grip_bounds = grip_bounds(bounds);
            let grip_hover =
                !state.is_resizing && cursor.position().is_some_and(|p| grip_bounds.contains(p));
            let color = if state.is_resizing || grip_hover {
                Color::from_rgba(0.9, 0.9, 1.0, 1.0)
            } else {
                Color::from_rgba(0.7, 0.7, 0.82, 0.8)
            };

            renderer.fill_quad(
                Quad {
                    bounds: grip_bounds,
                    border: border::rounded(3.0),
                    ..Default::default()
                },
                Color::from_rgba(0.08, 0.08, 0.14, 0.95),
            );

            for step in 0..3 {
                let offset = step as f32 * 4.0;
                renderer.fill_quad(
                    Quad {
                        bounds: Rectangle {
                            x: grip_bounds.x + grip_bounds.width - 4.0 - offset,
                            y: grip_bounds.y + grip_bounds.height - 4.0 - offset,
                            width: 2.5,
                            height: 2.5,
                        },
                        ..Default::default()
                    },
                    color,
                );
            }
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();

        let title_bounds = Rectangle {
            x: bounds.x,
            y: bounds.y,
            width: bounds.width,
            height: TITLE_BAR_HEIGHT,
        };

        let grip_bounds = grip_bounds(bounds);

        if state.is_resizing {
            return mouse::Interaction::ResizingDiagonallyDown;
        }

        if state.is_dragging {
            return mouse::Interaction::Grabbing;
        }

        if let Some(cursor_pos) = cursor.position() {
            if self.resizable && grip_bounds.contains(cursor_pos) {
                return mouse::Interaction::ResizingDiagonallyDown;
            }

            if title_bounds.contains(cursor_pos) {
                return mouse::Interaction::Grab;
            }
        }

        mouse::Interaction::None
    }

    fn overlay<'b>(
        &'b mut self,
        _tree: &'b mut Tree,
        _layout: Layout<'b>,
        _renderer: &Renderer,
        _viewport: &Rectangle,
        _translation: Vector,
    ) -> Vec<overlay::Element<'b, Message, Theme, Renderer>> {
        Vec::new()
    }
}

impl<'a, Message, Theme, Renderer> From<FloatingPanel<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: 'a,
    Renderer: renderer::Renderer + text::Renderer + 'a,
{
    fn from(panel: FloatingPanel<'a, Message, Theme, Renderer>) -> Self {
        Element::new(panel)
    }
}

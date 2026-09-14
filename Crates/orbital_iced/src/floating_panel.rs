use iced_core::event::Event;
use iced_core::layout::{self, Layout, Node};
use iced_core::mouse;
use iced_core::overlay;
use iced_core::renderer;
use iced_core::shell::Shell;
use iced_core::widget::{self, Tree};
use iced_core::{Element, Length, Point, Rectangle, Size, Vector};

const TITLE_BAR_HEIGHT: f32 = 28.0;
const DRAG_DEADBAND: f32 = 3.0;

#[derive(Debug, Clone)]
pub struct State {
    position: Point,
    is_dragging: bool,
    drag_origin: Point,
    drag_offset: Vector,
}

impl Default for State {
    fn default() -> Self {
        Self {
            position: Point::new(50.0, 50.0),
            is_dragging: false,
            drag_origin: Point::ORIGIN,
            drag_offset: Vector::new(0.0, 0.0),
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
    Renderer: renderer::Renderer + 'a,
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

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        _limits: &layout::Limits,
    ) -> Node {
        let state = tree.state.downcast_mut::<State>();

        let title_widget = self.title.as_widget_mut();
        let title_size = title_widget.layout(
            tree.children.get_mut(0).unwrap_or(&mut Tree::empty()),
            renderer,
            &layout::Limits::NONE,
        );

        let content_widget = self.content.as_widget_mut();
        let content_size = content_widget.layout(
            tree.children.get_mut(1).unwrap_or(&mut Tree::empty()),
            renderer,
            &layout::Limits::NONE,
        );

        let panel_width = title_size
            .size()
            .width
            .max(content_size.size().width)
            .max(200.0);
        let panel_height = TITLE_BAR_HEIGHT + content_size.size().height;

        Node::with_children(
            Size::new(panel_width, panel_height),
            vec![
                title_size.move_to(state.position),
                content_size.move_to(Point::new(
                    state.position.x,
                    state.position.y + TITLE_BAR_HEIGHT,
                )),
            ],
        )
    }

    fn diff(&mut self, tree: &mut Tree) {
        // Initialize state if tree was created via Cache::new() (State::None)
        if let widget::tree::State::None = &tree.state {
            tree.tag = self.tag();
            tree.state = self.state();
        }
        tree.children.clear();
        tree.children
            .push(Tree::new(self.title.as_widget_mut()));
        tree.children
            .push(Tree::new(self.content.as_widget_mut()));
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

        if let Event::Mouse(mouse_event) = event {
            match mouse_event {
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    if let Some(cursor_pos) = cursor.position() {
                        if title_bounds.contains(cursor_pos)
                            && !close_bounds.contains(cursor_pos)
                        {
                            state.is_dragging = true;
                            state.drag_origin = cursor_pos;
                            state.drag_offset = cursor_pos - state.position;
                            shell.capture_event();
                        } else if close_bounds.contains(cursor_pos) {
                            if let Some(msg) = self.on_close.clone() {
                                shell.publish(msg);
                                shell.capture_event();
                            }
                        }
                    }
                }
                mouse::Event::ButtonReleased(mouse::Button::Left) => {
                    if state.is_dragging {
                        state.is_dragging = false;
                        shell.capture_event();
                    }
                }
                mouse::Event::CursorMoved { position } => {
                    if state.is_dragging {
                        let delta = *position - state.drag_origin;
                        let dist = (delta.x * delta.x + delta.y * delta.y).sqrt();
                        let offset_dist =
                            (state.drag_offset.x * state.drag_offset.x
                                + state.drag_offset.y * state.drag_offset.y)
                                .sqrt();
                        if dist > DRAG_DEADBAND || offset_dist > 0.0 {
                            state.position = *position - state.drag_offset;
                            shell.request_redraw();
                            shell.capture_event();
                        }
                    }
                }
                _ => {}
            }
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

        // Forward events to content child
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
        // Draw title
        let title_layout = layout.child(0);
        self.title.as_widget().draw(
            tree.children.get(0).unwrap(),
            renderer,
            theme,
            style,
            title_layout,
            cursor,
            viewport,
        );

        // Draw content
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

        if state.is_dragging {
            return mouse::Interaction::Grabbing;
        }

        if let Some(cursor_pos) = cursor.position() {
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
    Renderer: renderer::Renderer + 'a,
{
    fn from(panel: FloatingPanel<'a, Message, Theme, Renderer>) -> Self {
        Element::new(panel)
    }
}

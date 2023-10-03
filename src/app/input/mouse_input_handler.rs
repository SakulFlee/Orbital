use winit::{
    dpi::{LogicalPosition, PhysicalPosition},
    event::{ElementState, MouseButton, MouseScrollDelta, TouchPhase},
    window::{CursorGrabMode, Window},
};

#[derive(Debug)]
pub struct MouseInputHandler {
    cursor_x: f64,
    cursor_y: f64,
    is_inside: bool,
    lmb_pressed: bool,
    rmb_pressed: bool,
    mmb_pressed: bool,
    scroll: Option<(TouchPhase, MouseScrollDelta)>,
    is_grabbed: bool,
    should_grab: bool,
    hide_mouse_if_grabbed: bool,
    reset_cursor_to_center: bool,
}

impl MouseInputHandler {
    pub fn new() -> Self {
        Self {
            cursor_x: 0.0,
            cursor_y: 0.0,
            is_inside: false,
            lmb_pressed: false,
            rmb_pressed: false,
            mmb_pressed: false,
            scroll: None,
            is_grabbed: false,
            should_grab: true,
            hide_mouse_if_grabbed: true,
            reset_cursor_to_center: true,
        }
    }

    fn post_update_grabbing(&mut self, window: &mut Window) {
        if self.should_grab && !self.is_grabbed {
            let result = window
                .set_cursor_grab(CursorGrabMode::Confined)
                .or_else(|_| window.set_cursor_grab(CursorGrabMode::Locked));

            if result.is_err() {
                log::warn!("Failed grabbing cursor: {:?}", result.unwrap_err());
                self.is_grabbed = false;
            } else {
                self.is_grabbed = true;
            }

            if self.hide_mouse_if_grabbed {
                window.set_cursor_visible(!self.is_grabbed);
            } else {
                window.set_cursor_visible(true);
            }
        } else if !self.should_grab && self.is_grabbed {
            let result = window.set_cursor_grab(CursorGrabMode::None);

            if result.is_err() {
                log::warn!("Failed un-grabbing cursor: {:?}", result.unwrap_err());
                self.is_grabbed = true;
            } else {
                self.is_grabbed = false;
            }

            if self.hide_mouse_if_grabbed {
                window.set_cursor_visible(!self.is_grabbed);
            } else {
                window.set_cursor_visible(true);
            }
        }
    }

    pub fn post_update_cursor_position(&mut self, window: &mut Window) {
        if !self.reset_cursor_to_center {
            return;
        }

        let window_size = window.inner_size();
        if let Err(e) = window.set_cursor_position(LogicalPosition::new(
            window_size.width / 2,
            window_size.height / 2,
        )) {
            log::warn!("Failed resetting cursor: {:?}", e);
        }
    }

    pub fn post_update(&mut self, window: &mut Window) {
        self.post_update_grabbing(window);
        self.post_update_cursor_position(window);
    }

    pub fn handle_cursor_moved(&mut self, position: PhysicalPosition<f64>) {
        self.cursor_x = position.x;
        self.cursor_y = position.y;
    }

    pub fn handle_cursor_entered(&mut self) {
        self.is_inside = true;
    }

    pub fn handle_cursor_left(&mut self) {
        self.is_inside = false;
    }

    pub fn handle_mouse_input(&mut self, state: ElementState, button: MouseButton) {
        match button {
            MouseButton::Left => {
                self.lmb_pressed = state == ElementState::Pressed;
            }
            MouseButton::Right => {
                self.rmb_pressed = state == ElementState::Pressed;
            }
            MouseButton::Middle => {
                self.mmb_pressed = state == ElementState::Pressed;
            }
            MouseButton::Other(_) => (),
        }
    }

    pub fn handle_mouse_scroll(&mut self, phase: TouchPhase, delta: MouseScrollDelta) {
        self.scroll = Some((phase, delta));
    }

    pub fn cursor_position(&self) -> (f64, f64) {
        (self.cursor_x, self.cursor_y)
    }

    pub fn is_inside(&self) -> bool {
        self.is_inside
    }

    pub fn lmb_pressed(&self) -> bool {
        self.lmb_pressed
    }

    pub fn rmb_pressed(&self) -> bool {
        self.rmb_pressed
    }

    pub fn mmb_pressed(&self) -> bool {
        self.mmb_pressed
    }

    pub fn scroll(&self) -> Option<(TouchPhase, MouseScrollDelta)> {
        self.scroll
    }

    pub fn is_grabbed(&self) -> bool {
        self.is_grabbed
    }

    pub fn should_grab(&self) -> bool {
        self.should_grab
    }

    pub fn set_should_grab(&mut self, should_grab: bool) {
        self.should_grab = should_grab;
    }

    pub fn hide_mouse_if_grabbed(&self) -> bool {
        self.hide_mouse_if_grabbed
    }

    pub fn set_hide_mouse_if_grabbed(&mut self, hide_mouse_if_grabbed: bool) {
        self.hide_mouse_if_grabbed = hide_mouse_if_grabbed;
    }

    pub fn reset_cursor_to_center(&self) -> bool {
        self.reset_cursor_to_center
    }

    pub fn set_reset_cursor_to_center(&mut self, reset_cursor_to_center: bool) {
        self.reset_cursor_to_center = reset_cursor_to_center;
    }
}

impl Default for MouseInputHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for MouseInputHandler {
    fn default() -> Self {
        Self::new()
    }
}

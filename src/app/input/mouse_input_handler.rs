use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, MouseButton, MouseScrollDelta, TouchPhase},
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
        }
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
}

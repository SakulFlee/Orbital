use crate::state::IcedState;
use iced_winit::winit::event::{ElementState, MouseButton, WindowEvent};

pub enum IcedEvent {
    None,
    ButtonPressed(String),
    CursorMoved(f64, f64),
    Resized(f32, f32),
}

pub struct IcedEventBridge {
    state: IcedState,
}

impl IcedEventBridge {
    pub fn new(state: IcedState) -> Self {
        Self { state }
    }

    pub fn process_event(&self, event: &WindowEvent) -> Option<IcedEvent> {
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    log::debug!("Key pressed: {:?}", event.logical_key);
                }
                None
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if *state == ElementState::Pressed {
                    match button {
                        MouseButton::Left => Some(IcedEvent::ButtonPressed("left_click".to_string())),
                        MouseButton::Right => Some(IcedEvent::ButtonPressed("right_click".to_string())),
                        MouseButton::Middle => Some(IcedEvent::ButtonPressed("middle_click".to_string())),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                Some(IcedEvent::CursorMoved(position.x, position.y))
            }
            WindowEvent::Resized(size) => {
                Some(IcedEvent::Resized(size.width as f32, size.height as f32))
            }
            _ => None,
        }
    }

    pub fn state(&self) -> &IcedState {
        &self.state
    }
}

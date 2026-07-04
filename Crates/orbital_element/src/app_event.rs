use crate::Message;
use winit::{dpi::Position, window::Cursor};

#[derive(Debug)]
pub enum AppEvent {
    ChangeCursorAppearance(Cursor),
    ChangeCursorPosition(Position),
    ChangeCursorVisible(bool),
    ChangeCursorGrabbed(bool),
    RequestAppClosure,
    ForceAppClosure { exit_code: i32 },
    RequestRedraw,
    SendMessage(Message),
}

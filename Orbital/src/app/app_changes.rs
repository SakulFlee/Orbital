use winit::{dpi::Position, window::Cursor};

pub enum AppChange {
    /// Changes the appearance (i.e. icon) of the mouse cursor.  
    /// Gets send directly to [winit], issues may appear in log.
    ChangeCursorAppearance(Cursor),
    /// Changes the mouse cursor position.  
    /// Gets send directly to [winit], issues may appear in log.
    ChangeCursorPosition(Position),
    /// Changes the mouse cursor visibility.  
    /// `true` means the cursor will be visible, whereas `false` means invisible.  
    /// Gets send directly to [winit], issues may appear in log.
    ChangeCursorVisible(bool),
    /// Changes if the mouse cursor should be grabbed or not.  
    /// A grabbed mouse cursor **cannot** escape the current window.  
    /// Gets send directly to [winit], issues may appear in log.
    ChangeCursorGrabbed(bool),
}

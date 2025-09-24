use crate::element::Message;
use winit::{dpi::Position, window::Cursor};

#[derive(Debug)]
pub enum AppEvent {
    /// Changes the appearance (i.e. icon) of the mouse cursor.  
    /// Gets send directly to [winit], issues may appear in log.
    ChangeCursorAppearance(Cursor),
    /// Changes the mouse cursor position.  
    /// Gets send directly to [winit], issues may appear in log.
    ///
    /// Check [Window::set_cursor_position](winit::window::Window::set_cursor_position) for more information and compatibility.
    ChangeCursorPosition(Position),
    /// Changes the mouse cursor visibility.  
    /// `true` means the cursor will be visible, whereas `false` means invisible.  
    /// Gets send directly to [winit], issues may appear in log.
    ///
    /// Check [Window::set_cursor_visible](winit::window::Window::set_cursor_visible) for more information and compatibility.
    ChangeCursorVisible(bool),
    /// Changes if the mouse cursor should be grabbed or not.  
    /// A grabbed mouse cursor **cannot** escape the current window.  
    /// Gets send directly to [winit], issues may appear in log.
    ///
    /// Check [Window::set_cursor_grab](winit::window::Window::set_cursor_grab) for more information and compatibility.
    ChangeCursorGrabbed(bool),
    /// Requested that the app will close itself as soon as possible.
    /// The internal event loop will be stopped and the window will be closed.
    /// If there are other child-threads or processes active, they _may_ remain.
    /// It may take some time until everything is resolved and exits, this is a
    /// "graceful exit".
    ///
    /// The main process will exit with "exit code zero (0)".
    ///
    /// See also: [AppChange::ForceAppClosure]
    RequestAppClosure,
    /// Same as [AppChange::RequestAppClosure], but will force the app to exit.
    /// Any child-threads will be killed along the way by the operating system.
    /// This will attempt to immediately exit the app, without leaving time for
    /// cleanup or "graceful exits".
    ///
    /// The main process will exit with the defined "exit code".
    ///
    /// See also: [AppChange::RequestAppClosure]
    ForceAppClosure { exit_code: i32 },
    /// Used to signal the App to perform a redraw cycle.
    /// If the `auto_request_redraw` feature is enabled, you will never need to use this.
    /// If the `auto_request_redraw` feature is disabled, you need to send this event every time you want to redraw the screen (surface).
    ///
    /// This does not immediately force a redraw, but instead queue it to be executed as soon as possible. Thus, there might be some slight delay between sending this event and the redraw happening.
    RequestRedraw,
    /// Sends a message to the App instead of an Element.
    /// Useful for communicating with the overall app to change fundamental settings like, for example, which renderer to use.
    SendMessage(Message),
}

use std::sync::{Arc, Mutex};

use crate::AppContext;

#[derive(Debug)]
pub enum AppState {
    Starting,
    Ready(Arc<Mutex<AppContext>>),
    /// The app was backgrounded (activity paused) but the native window /
    /// context is kept alive so it can be resumed without recreating the
    /// window (mirrors Bevy, which reuses the winit Window across suspend).
    Paused(Arc<Mutex<AppContext>>),
    Ending,
}

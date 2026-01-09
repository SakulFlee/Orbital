use std::sync::{Arc, Mutex};

use crate::app::AppContext;

#[derive(Debug)]
pub enum AppState {
    Starting,
    Ready(Arc<Mutex<AppContext>>),
    Paused,
    Ending,
}

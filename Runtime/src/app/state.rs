use crate::app::AppContext;

#[derive(Debug)]
pub enum AppState {
    Starting,
    Ready(AppContext),
    Paused,
    Ending,
}

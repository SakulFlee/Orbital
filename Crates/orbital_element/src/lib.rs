pub mod app_event;
pub use app_event::*;

pub mod registration;
pub use registration::*;

pub mod store;
pub use store::*;

pub mod message;
pub use message::*;

mod event;
pub use event::*;

use log::info;
use orbital_input::InputState;
use std::fmt::Debug;
use std::sync::Arc;

pub trait Element: Debug + Send {
    fn on_registration(&self) -> ElementRegistration;

    fn on_message(&mut self, message: &Arc<Message>) -> Option<Vec<Event>> {
        if let Target::Element { .. } = message.to() {
            info!("Received message that isn't handled: {message:?}");
        }

        None
    }

    fn on_update(&mut self, _delta_time: f64, _input_state: &InputState) -> Option<Vec<Event>> {
        None
    }
}

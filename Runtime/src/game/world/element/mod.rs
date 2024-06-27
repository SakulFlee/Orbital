use hashbrown::HashMap;
use log::warn;
use ulid::Ulid;

use crate::{game::WorldChange, variant::Variant};

pub mod registration;
pub use registration::*;

pub trait Element {
    fn on_registration(&mut self, _ulid: &Ulid) -> ElementRegistration {
        ElementRegistration::default()
    }

    fn on_update(&mut self, _delta_time: f64) -> Option<Vec<WorldChange>> {
        None
    }

    fn on_message(&mut self, message: HashMap<String, Variant>) -> Option<Vec<WorldChange>> {
        warn!("Unhandled message received: {:#?}", message);

        None
    }
}

#[macro_use]
extern crate console_log;

#[cfg(test)]
mod tests;

pub mod event;
pub use event::*;

pub mod events;
pub use events::*;

pub mod event_system;
pub use event_system::*;

pub mod boxed_event;
pub use boxed_event::*;

pub mod singleton;
pub use singleton::*;

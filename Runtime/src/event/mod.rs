#![allow(clippy::module_inception)]

#[cfg(test)]
mod tests;

pub mod event;
pub use event::*;

pub mod event_system;
pub use event_system::*;

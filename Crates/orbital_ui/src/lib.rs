//! # Orbital UI
//!
//! UI system for the Orbital engine.
//!
//! Provides:
//! - UI components (Button, Text, TextBox, Checkbox, Image)
//! - Layout system (absolute, horizontal, vertical)
//! - Event system for UI interactions
//! - Hit testing for mouse/touch input
//!
//! ## Quick Start
//!
//! ```ignore
//! use orbital_ui::{create_button, UiLayout, UiElement};
//!
//! fn setup_ui(world: &mut World) {
//!     let button = create_button(world, "play_btn", "Play", UiLayout::Absolute {
//!         x: 100.0,
//!         y: 200.0,
//!         width: 200.0,
//!         height: 50.0,
//!     });
//! }
//! ```

pub mod components;
pub mod events;
pub mod hit_test;
pub mod layout;
pub mod widgets;

pub use components::*;
pub use events::*;
pub use hit_test::{HitResult, hit_test, hit_test_all};
pub use layout::{Alignment, UiLayout};

#[cfg(test)]
mod tests {
    use super::*;
    use orbital_ecs::World;

    #[test]
    fn ui_setup_integration() {
        let mut world = World::new();

        // Create a button
        let btn = widgets::create_button(&mut world, "btn1", "Click Me", UiLayout::default());
        assert!(world.get_component_store::<UiButton>().is_some());

        // Create text
        let txt = widgets::create_text(&mut world, "txt1", "Hello", UiLayout::default());
        assert!(world.get_component_store::<UiText>().is_some());

        // Create checkbox
        let cb = widgets::create_checkbox(&mut world, "cb1", "Accept", UiLayout::default());
        assert!(world.get_component_store::<UiCheckbox>().is_some());
    }

    #[test]
    fn events_workflow() {
        use orbital_ecs::Events;

        let mut events = Events::<ButtonPressed>::new();
        let entity = orbital_ecs::Entity::new(0, 0);

        // Send event
        events.send(ButtonPressed {
            entity,
            id: "play".into(),
        });

        // Read event
        let collected: Vec<_> = events.read().collect();
        assert_eq!(collected.len(), 1);
        assert_eq!(collected[0].id, "play");

        // Clear events
        events.clear();
        assert!(events.is_empty());
    }
}

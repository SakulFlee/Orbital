use orbital_ecs::{Entity, World};

use crate::components::{ButtonState, UiBackground, UiButton, UiElement, UiEntity, UiFocus};
use crate::layout::UiLayout;

/// Creates a button entity with the given label and layout.
pub fn create_button(
    world: &mut World,
    id: impl Into<String>,
    label: impl Into<String>,
    layout: UiLayout,
) -> Entity {
    let entity = world.spawn_entity();

    world
        .attach_component(&entity, UiEntity)
        .expect("Failed to attach UiEntity");
    world
        .attach_component(&entity, UiElement::new(id))
        .expect("Failed to attach UiElement");
    world
        .attach_component(&entity, UiButton::new(label))
        .expect("Failed to attach UiButton");
    world
        .attach_component(&entity, layout)
        .expect("Failed to attach UiLayout");
    world
        .attach_component(&entity, UiBackground::rounded([0.3, 0.3, 0.3, 0.9], 4.0))
        .expect("Failed to attach UiBackground");
    world
        .attach_component(&entity, UiFocus::default())
        .expect("Failed to attach UiFocus");

    entity
}

/// Updates the button state based on hover and press status.
pub fn update_button_state(button: &mut UiButton, hovered: bool, pressed: bool) {
    button.state = if pressed {
        ButtonState::Pressed
    } else if hovered {
        ButtonState::Hovered
    } else {
        ButtonState::Normal
    };
}

/// Returns the button color based on its state.
pub fn button_color(state: ButtonState) -> [f32; 4] {
    match state {
        ButtonState::Normal => [0.3, 0.3, 0.3, 0.9],
        ButtonState::Hovered => [0.4, 0.4, 0.4, 0.9],
        ButtonState::Pressed => [0.2, 0.2, 0.2, 0.9],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_button_entity() {
        let mut world = World::new();
        let entity = create_button(&mut world, "btn1", "Click Me", UiLayout::default());

        assert!(world.get_component_store::<UiEntity>().is_some());
        assert!(world.get_component_store::<UiButton>().is_some());
    }

    #[test]
    fn button_state_update() {
        let mut button = UiButton::new("Test");

        update_button_state(&mut button, false, false);
        assert_eq!(button.state, ButtonState::Normal);

        update_button_state(&mut button, true, false);
        assert_eq!(button.state, ButtonState::Hovered);

        update_button_state(&mut button, true, true);
        assert_eq!(button.state, ButtonState::Pressed);
    }

    #[test]
    fn button_colors() {
        assert_eq!(button_color(ButtonState::Normal), [0.3, 0.3, 0.3, 0.9]);
        assert_eq!(button_color(ButtonState::Hovered), [0.4, 0.4, 0.4, 0.9]);
        assert_eq!(button_color(ButtonState::Pressed), [0.2, 0.2, 0.2, 0.9]);
    }
}

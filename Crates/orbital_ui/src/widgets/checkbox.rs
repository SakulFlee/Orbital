use orbital_ecs::{Entity, World};

use crate::components::{UiBackground, UiCheckbox, UiElement, UiEntity, UiFocus};
use crate::layout::UiLayout;

/// Creates a checkbox entity with the given label and layout.
pub fn create_checkbox(
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
        .attach_component(&entity, UiCheckbox::new(label))
        .expect("Failed to attach UiCheckbox");
    world
        .attach_component(&entity, layout)
        .expect("Failed to attach UiLayout");
    world
        .attach_component(&entity, UiBackground::solid([0.2, 0.2, 0.2, 0.9]))
        .expect("Failed to attach UiBackground");
    world
        .attach_component(&entity, UiFocus::default())
        .expect("Failed to attach UiFocus");

    entity
}

/// Toggles the checkbox state.
pub fn toggle_checkbox(checkbox: &mut UiCheckbox) {
    checkbox.checked = !checkbox.checked;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_checkbox_entity() {
        let mut world = World::new();
        let entity = create_checkbox(&mut world, "cb1", "Accept Terms", UiLayout::default());

        assert!(world.get_component_store::<UiEntity>().is_some());
        assert!(world.get_component_store::<UiCheckbox>().is_some());
    }

    #[test]
    fn toggle_checkbox_test() {
        let mut checkbox = UiCheckbox::new("Test");
        assert!(!checkbox.checked);

        toggle_checkbox(&mut checkbox);
        assert!(checkbox.checked);

        toggle_checkbox(&mut checkbox);
        assert!(!checkbox.checked);
    }
}

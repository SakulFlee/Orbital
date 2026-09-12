use orbital_ecs::{Entity, World};

use crate::components::{UiElement, UiEntity, UiText};
use crate::layout::UiLayout;

/// Creates a text entity with the given content and layout.
pub fn create_text(
    world: &mut World,
    id: impl Into<String>,
    content: impl Into<String>,
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
        .attach_component(&entity, UiText::new(content))
        .expect("Failed to attach UiText");
    world
        .attach_component(&entity, layout)
        .expect("Failed to attach UiLayout");

    entity
}

/// Creates a styled text entity.
pub fn create_styled_text(
    world: &mut World,
    id: impl Into<String>,
    content: impl Into<String>,
    font_size: f32,
    color: [f32; 4],
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
        .attach_component(
            &entity,
            UiText::new(content).with_font_size(font_size).with_color(color),
        )
        .expect("Failed to attach UiText");
    world
        .attach_component(&entity, layout)
        .expect("Failed to attach UiLayout");

    entity
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_text_entity() {
        let mut world = World::new();
        let entity = create_text(&mut world, "txt1", "Hello World", UiLayout::default());

        assert!(world.get_component_store::<UiEntity>().is_some());
        assert!(world.get_component_store::<UiText>().is_some());
    }

    #[test]
    fn create_styled_text_entity() {
        let mut world = World::new();
        let entity = create_styled_text(
            &mut world,
            "txt2",
            "Styled",
            32.0,
            [1.0, 0.0, 0.0, 1.0],
            UiLayout::default(),
        );

        let store = world.get_component_store::<UiText>().unwrap();
        let text = store.get_component(entity.index).unwrap();
        assert_eq!(text.font_size, 32.0);
        assert_eq!(text.color, [1.0, 0.0, 0.0, 1.0]);
    }
}

use orbital_ecs::{Entity, World};

use crate::components::{UiElement, UiEntity, UiImage};
use crate::layout::UiLayout;

/// Creates an image entity with the given layout.
pub fn create_image(
    world: &mut World,
    id: impl Into<String>,
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
        .attach_component(&entity, UiImage::new())
        .expect("Failed to attach UiImage");
    world
        .attach_component(&entity, layout)
        .expect("Failed to attach UiLayout");

    entity
}

/// Creates an image entity with a tint color.
pub fn create_tinted_image(
    world: &mut World,
    id: impl Into<String>,
    tint: [f32; 4],
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
        .attach_component(&entity, UiImage::new().with_tint(tint))
        .expect("Failed to attach UiImage");
    world
        .attach_component(&entity, layout)
        .expect("Failed to attach UiLayout");

    entity
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_image_entity() {
        let mut world = World::new();
        let entity = create_image(&mut world, "img1", UiLayout::default());

        assert!(world.get_component_store::<UiEntity>().is_some());
        assert!(world.get_component_store::<UiImage>().is_some());
    }

    #[test]
    fn create_tinted_image_entity() {
        let mut world = World::new();
        let entity = create_tinted_image(&mut world, "img2", [1.0, 0.0, 0.0, 1.0], UiLayout::default());

        let store = world.get_component_store::<UiImage>().unwrap();
        let image = store.get_component(entity.index).unwrap();
        assert_eq!(image.tint, [1.0, 0.0, 0.0, 1.0]);
    }
}

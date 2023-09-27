use wgpu_engine::{
    app::{App, EntityTagDuplicationBehaviour, World, WorldBuilder},
    engine::rgb_to_f32_color,
    entities::{CameraControllingEntity, ClearScreenEntity, Square},
    log::log_init,
};

fn main() {
    log_init();

    let world_builder = WorldBuilder::new()
        .with_clear_color(World::SKY_BLUE_ISH_COLOR)
        .with_entity_tag_duplication_behaviour(EntityTagDuplicationBehaviour::WarnOnDuplication)
        .with_ambient_light(rgb_to_f32_color(255u8, 50u8, 50u8), 0.1)
        .with_entities(vec![
            Box::new(CameraControllingEntity::new()),
            Box::new(ClearScreenEntity {}),
            Box::<Square>::default(),
        ]);

    App::run("WGPU", world_builder).expect("App failed");
}

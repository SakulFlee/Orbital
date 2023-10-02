use cgmath::Vector3;
use wgpu::Color;
use wgpu_engine::{
    app::{App, EntityTagDuplicationBehaviour, WorldBuilder},
    engine::rgb_to_f32_color,
    entities::{BrickCube, CameraControllingEntity, Cheese, ClearScreenEntity, Square},
    log::log_init,
};

fn main() {
    log_init();

    let world_builder = WorldBuilder::new()
        // .with_clear_color(World::SKY_BLUE_ISH_COLOR)
        .with_clear_color(Color::BLACK) // TODO: Not working
        .with_entity_tag_duplication_behaviour(EntityTagDuplicationBehaviour::WarnOnDuplication)
        // .with_ambient_light(rgb_to_f32_color(255u8, 50u8, 50u8), 0.25)
        .with_ambient_light(rgb_to_f32_color(255u8, 255u8, 255u8), 0.1)
        .with_point_light(
            // TODO: Spawn cube for debugging at location
            0,
            rgb_to_f32_color(255u8, 255u8, 255u8).into(),
            Vector3::new(0.0, 2.5, 0.0),
            5.0,
        )
        .with_entities(vec![
            Box::new(CameraControllingEntity::new()),
            Box::new(ClearScreenEntity {}),
            Box::<Square>::default(),
            Box::<BrickCube>::default(),
        ]);

    App::run("WGPU", world_builder).expect("App failed");
}

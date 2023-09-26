use wgpu_engine::{
    app::{App, World},
    entities::{CameraControllingEntity, ClearScreenEntity, Square},
    log::log_init,
};

fn main() {
    log_init();

    let mut world = World::new();

    world.add_entity(Box::new(CameraControllingEntity::new()));
    world.add_entity(Box::new(ClearScreenEntity {}));
    world.add_entity(Box::new(Square::default()));

    App::run("WGPU", world).expect("App failed");
}

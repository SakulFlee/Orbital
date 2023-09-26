use wgpu_engine::{
    app::{App, EntityTagDuplicationBehaviour, World},
    entities::{CameraControllingEntity, ClearScreenEntity, Square},
    log::log_init,
};

fn main() {
    log_init();

    let mut world = World::new(EntityTagDuplicationBehaviour::PanicOnDuplication);

    world.add_entity(Box::new(CameraControllingEntity::new()));
    world.add_entity(Box::new(ClearScreenEntity {}));
    world.add_entity(Box::new(Square::default()));

    App::run("WGPU", world).expect("App failed");
}

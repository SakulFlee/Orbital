use wgpu_engine::{
    app::{App, World},
    entities::{ClearScreenEntity, Square},
    log::log_init,
};

fn main() {
    log_init();

    let mut world = World::new();

    world.add_entity(Box::new(Square::default()));
    world.add_entity(Box::new(ClearScreenEntity {}));

    App::run("WGPU", world).expect("App failed");
}

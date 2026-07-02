use orbital_ecs::World;

// ---------------------------------------------------------------------------
// Component types for benchmarks
// ---------------------------------------------------------------------------

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Pos(pub f32, pub f32);

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Vel(pub f32, pub f32);

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Name(pub String);

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Score(pub i32);

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Frozen;

// ---------------------------------------------------------------------------
// World factories
// ---------------------------------------------------------------------------

pub fn setup_dense_world(n: usize) -> World {
    let mut world = World::new();
    for i in 0..n {
        let e = world.spawn_entity();
        world.attach_component(&e, Pos(i as f32, 0.0)).unwrap();
        world.attach_component(&e, Vel(1.0, 0.0)).unwrap();
        world.attach_component(&e, Name(format!("e{i}"))).unwrap();
        world.attach_component(&e, Score(i as i32)).unwrap();
    }
    world
}

pub fn setup_partial_world(n: usize) -> World {
    let mut world = World::new();
    for i in 0..n {
        let e = world.spawn_entity();
        world.attach_component(&e, Pos(i as f32, 0.0)).unwrap();
        if i % 2 == 0 {
            world.attach_component(&e, Vel(1.0, 0.0)).unwrap();
        }
    }
    world
}

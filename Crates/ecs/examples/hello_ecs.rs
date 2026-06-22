/// Minimal ECS example.
///
/// Run: cargo run --example hello_ecs --release
use orbital_ecs::{Commands, Schedule, World};

#[derive(Debug, Clone)]
struct Name(String);

#[derive(Debug, Clone)]
struct Health(i32);

fn heal(health: &mut Health) {
    health.0 = (health.0 + 10).min(100);
}

fn main() {
    let mut world = World::new();
    let e1 = world.spawn_entity();
    world.attach_component(&e1, Name("Alice".into())).unwrap();
    world.attach_component(&e1, Health(30)).unwrap();

    let e2 = world.spawn_entity();
    world.attach_component(&e2, Name("Bob".into())).unwrap();
    world.attach_component(&e2, Health(80)).unwrap();

    // Run schedule with one system.
    let mut schedule = Schedule::new();
    schedule.add_system(heal);
    schedule.run(&world);

    // Read back via store handles.
    for e in &[e1, e2] {
        let store = world.get_component_store::<Health>().unwrap();
        let name_store = world.get_component_store::<Name>().unwrap();
        let h = store.get_component(e.index).unwrap();
        let n = name_store.get_component(e.index).unwrap();
        println!("  {} → HP: {}", n.0, h.0);
    }

    // Commands: deferred entity spawn.
    let mut cmds = Commands::new();
    let e3 = cmds.spawn_entity();
    cmds.attach_component(&e3, Name("Charlie".into()));
    cmds.attach_component(&e3, Health(50));
    cmds.flush(&mut world).unwrap();

    let store = world.get_component_store::<Name>().unwrap();
    let health_store = world.get_component_store::<Health>().unwrap();
    for e in &[e3] {
        let n = store.get_component(e.index).unwrap();
        let h = health_store.get_component(e.index).unwrap();
        println!("  {} → HP: {}", n.0, h.0);
    }
}

//! Deferred entity spawning via Commands buffer.

use orbital_ecs::{Commands, Schedule, World};

#[derive(Debug, Clone)]
struct Name(String);

#[derive(Debug, Clone)]
struct Health(i32);

fn heal(health: &mut Health) {
    health.0 = (health.0 + 10).min(100);
}

fn print_all(name: &Name, health: &Health) {
    println!("  {} → HP: {}", name.0, health.0);
}

fn main() {
    let mut world = World::new();

    let e1 = world.spawn_entity();
    world.attach_component(&e1, Name("Alice".into())).unwrap();
    world.attach_component(&e1, Health(30)).unwrap();

    // Phase 1 — heal Alice and print.
    let mut schedule = Schedule::new();
    schedule.add_system(heal);
    schedule.add_system(print_all);
    schedule.run(&world);

    // Phase 2 — defer-spawn Charlie, then flush and run again.
    let mut cmds = Commands::new();
    let e2 = cmds.spawn_entity();
    cmds.attach_component(&e2, Name("Charlie".into()));
    cmds.attach_component(&e2, Health(50));
    cmds.flush(&mut world).unwrap();

    // Charlie is now visible to the schedule.
    schedule.run(&world);
}

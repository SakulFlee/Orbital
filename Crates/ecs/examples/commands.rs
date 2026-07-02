//! Deferred entity spawning via Commands buffer, driven by a system.
//!
//! Run: cargo run --example commands --release
use orbital_ecs::{Commands, ResMut, Schedule, World};

#[derive(Debug, Clone)]
struct Name(String);

#[derive(Debug, Clone)]
struct Health(i32);

#[derive(Debug, Clone)]
struct SpawnPlayers(bool);

fn heal(health: &mut Health) {
    health.0 = (health.0 + 10).min(100);
}

fn print_all(name: &Name, health: &Health) {
    println!("  {} → HP: {}", name.0, health.0);
}

fn spawn_players(mut commands: ResMut<Commands>, mut flag: ResMut<SpawnPlayers>) {
    if flag.0 {
        let e2 = commands.spawn_entity();
        commands.attach_component(&e2, Name("Charlie".into()));
        commands.attach_component(&e2, Health(50));

        let e3 = commands.spawn_entity();
        commands.attach_component(&e3, Name("Diana".into()));
        commands.attach_component(&e3, Health(20));

        flag.0 = false;
    }
}

fn main() {
    let mut world = World::new();

    world.insert_resource(Commands::new());
    world.insert_resource(SpawnPlayers(true));

    let e1 = world.spawn_entity();
    world.attach_component(&e1, Name("Alice".into())).unwrap();
    world.attach_component(&e1, Health(30)).unwrap();

    let mut schedule = Schedule::new();
    schedule.add_system(heal);
    schedule.add_system(spawn_players);
    schedule.add_system(print_all);

    // Run 1: heal Alice (30→40), queue Charlie + Diana, print Alice
    schedule.run(&world);

    // Flush — take Commands out of the resource, drop the lock, then flush
    let queued = std::mem::replace(
        &mut *world.get_resource_mut::<Commands>().unwrap(),
        Commands::new(),
    );
    queued.flush(&mut world).unwrap();

    // Run 2: heal all three, print all three
    schedule.run(&world);
}

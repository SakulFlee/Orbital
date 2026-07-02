//! # Minimal ECS Example
//! Systems do all the work.
//! The scheduler is run multiple times in a loop.
//! Each system runs on each run, modifying or interacting with the [`Health`] and [`Name`] component.

use orbital_ecs::{Res, ResMut, Schedule, World};

#[derive(Debug, Clone)]
struct Name(String);

#[derive(Debug, Clone)]
struct Health(i32);

#[derive(Debug, Clone)]
struct FrameCount(usize);

fn tick_frame(mut frame: ResMut<FrameCount>) {
    frame.0 += 1;
}

fn heal(health: &mut Health) {
    health.0 = (health.0 + 10).min(100);
}

fn print_frame(frame: Res<FrameCount>, name: &Name, health: &Health) {
    println!("#{}: {: <6} → HP: {: <2}", frame.0, name.0, health.0);
}

fn main() {
    let mut world = World::new();

    world.insert_resource(FrameCount(0));

    let e1 = world.spawn_entity();
    world.attach_component(&e1, Name("Alice".into())).unwrap();
    world.attach_component(&e1, Health(30)).unwrap();

    let e2 = world.spawn_entity();
    world.attach_component(&e2, Name("Bob".into())).unwrap();
    world.attach_component(&e2, Health(80)).unwrap();

    let mut schedule = Schedule::new();

    schedule.add_system(heal);
    schedule.add_system(print_frame);
    schedule.add_system(tick_frame);

    for _ in 0..9 {
        schedule.run(&world);
    }
}

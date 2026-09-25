//! # Interval (low priority) scheduling example
//!
//! A system can be marked to run less often than every update cycle.
//! `Schedule::run_with_time` drives the throttling: each system only runs
//! once its interval has elapsed on the given clock.
//!
//! Here the clock advances in 100 ms steps (simulating a 10 FPS frame loop
//! that would otherwise run every system ten times per second):
//!
//! - `scan` is low priority (`add_low_priority_system`) → runs 1× per second,
//! - `heartbeat` runs at an explicit 250 ms interval,
//! - a boxed system (as `Module::setup` would return) wrapped with
//!   `with_interval` → sweeps every 500 ms,
//! - `tick` is a normal system and runs every frame.
//!
//! Missed ticks are dropped, never replayed, so a hitch in the frame loop
//! can never cause a burst of catch-up runs.

use std::time::Duration;

use orbital_ecs::{IntoSystem, ResMut, Schedule, World, with_interval};

#[derive(Debug, Default)]
struct Clock {
    /// Schedule time in seconds, as passed to `run_with_time`.
    now: f64,
    scans: usize,
    heartbeats: usize,
    sweeps: usize,
    ticks: usize,
}

fn scan(mut clock: ResMut<Clock>) {
    clock.scans += 1;
    println!("[{:>5.2}s] scan (low priority, 1 Hz)", clock.now);
}

fn heartbeat(mut clock: ResMut<Clock>) {
    clock.heartbeats += 1;
    println!("[{:>5.2}s] heartbeat (every 250 ms)", clock.now);
}

fn tick(mut clock: ResMut<Clock>) {
    clock.ticks += 1;
}

fn main() {
    let mut world = World::new();
    world.insert_resource(Clock::default());

    let mut schedule = Schedule::new();

    // Normal system: runs on every update cycle.
    schedule.add_system(tick);

    // Low priority: runs once per second (10 frames at 10 FPS).
    schedule.add_low_priority_system(scan);

    // Explicit interval: runs once per 250 ms (every 2.5 frames).
    schedule.add_system_with_interval(heartbeat, Duration::from_millis(250));

    // Boxed systems (e.g. returned from `Module::setup`, which only hands
    // the runtime a `Vec<Box<dyn System>>`) can be wrapped instead of using
    // the Schedule-level helpers — this one sweeps every 500 ms.
    let boxed: Box<dyn orbital_ecs::System> = (|mut clock: ResMut<Clock>| {
        clock.sweeps += 1;
        println!("[{:>5.2}s] maintenance sweep (boxed, 500 ms)", clock.now);
    })
    .into_system();
    schedule.add_system_boxed(with_interval(boxed, Duration::from_millis(500)));

    // Simulate 2 seconds of a 10 FPS frame loop.
    for frame in 0..=20 {
        let now = frame as f64 * 0.1;

        if let Some(mut clock) = world.get_resource_mut::<Clock>() {
            clock.now = now;
        }

        schedule.run_with_time(&mut world, now);
    }

    let clock = world.get_resource::<Clock>().unwrap();
    println!(
        "\nAfter 2s at 10 FPS: {} ticks, {} scans, {} heartbeats, {} sweeps",
        clock.ticks, clock.scans, clock.heartbeats, clock.sweeps
    );
}

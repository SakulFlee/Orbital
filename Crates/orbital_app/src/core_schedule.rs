//! Core engine schedule — built-in systems that run every frame.
//!
//! These systems handle engine-level bookkeeping: frame timing, input
//! snapshots, and other state that the runtime writes into the ECS world
//! before user systems execute.

use orbital_ecs::ResMut;
use orbital_ecs_bridge::{DeltaTime, TotalTime};

/// Increments the frame counter each tick.
///
/// Runs in the core schedule, after the runtime has written the current
/// `DeltaTime` into the ECS world.
pub fn sys_accumulate_time(mut total: ResMut<TotalTime>, dt: ResMut<DeltaTime>) {
    total.0 += dt.0;
}

/// Returns the core engine systems as a pre-built schedule.
///
/// The runtime calls this once during initialisation and then runs it
/// at the start of every redraw cycle, before any user-defined systems.
pub fn make_core_schedule() -> orbital_ecs::Schedule {
    let mut schedule = orbital_ecs::Schedule::new();
    schedule.add_system(sys_accumulate_time);
    schedule
}

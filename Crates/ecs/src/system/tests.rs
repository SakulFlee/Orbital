use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use crate::World;
use crate::system::*;

#[derive(Debug, Clone)]
struct Pos(f32, f32);
#[derive(Debug, Clone)]
struct Vel(f32, f32);
#[derive(Debug)]
struct Frozen;

fn setup_world() -> World {
    let mut world = World::new();
    let e1 = world.spawn_entity();
    world.attach_component(&e1, Pos(0.0, 0.0)).unwrap();
    world.attach_component(&e1, Vel(1.0, 2.0)).unwrap();

    let e2 = world.spawn_entity();
    world.attach_component(&e2, Pos(10.0, 10.0)).unwrap();
    world.attach_component(&e2, Vel(-1.0, -1.0)).unwrap();

    let e3 = world.spawn_entity();
    world.attach_component(&e3, Pos(5.0, 5.0)).unwrap();
    world.attach_component(&e3, Frozen).unwrap();

    let e4 = world.spawn_entity();
    world.attach_component(&e4, Vel(100.0, 200.0)).unwrap();

    world
}

fn move_pos(pos: &mut Pos) {
    pos.0 += 1.0;
}

fn apply_vel(pos: &mut Pos, vel: &Vel) {
    pos.0 += vel.0;
    pos.1 += vel.1;
}

fn double_both(pos: &mut Pos, vel: &mut Vel) {
    pos.0 *= 2.0;
    vel.0 *= 2.0;
}

fn update_score(pos: &mut Pos, vel: &Vel, score: &mut i32) {
    pos.0 += vel.0;
    pos.1 += vel.1;
    *score += 1;
}

fn update_all(pos: &mut Pos, vel: &Vel, score: &mut i32, _flag: &bool) {
    pos.0 += vel.0;
    *score += 1;
}

fn add_five(pos: &mut Pos) {
    pos.0 += 5.0;
}

#[test]
fn system_write_single() {
    let mut world = setup_world();
    let mut schedule = Schedule::new();
    schedule.add_system(move_pos);
    schedule.run(&mut world);

    let store = world.get_component_store::<Pos>().unwrap();
    assert_eq!(store.get_component(0).unwrap().0, 1.0);
    assert_eq!(store.get_component(1).unwrap().0, 11.0);
    assert_eq!(store.get_component(2).unwrap().0, 6.0);
}

#[test]
fn system_read_single() {
    let mut world = setup_world();
    let count = Arc::new(AtomicUsize::new(0));
    {
        let c = Arc::clone(&count);
        let mut schedule = Schedule::new();
        schedule.add_system::<fn(&Pos), _>(move |_pos: &Pos| {
            c.fetch_add(1, Ordering::Relaxed);
        });
        schedule.run(&mut world);
    }
    assert_eq!(count.load(Ordering::Relaxed), 3);
}

#[test]
fn system_write_read_two() {
    let mut world = setup_world();
    let mut schedule = Schedule::new();
    schedule.add_system(apply_vel);
    schedule.run(&mut world);

    let store = world.get_component_store::<Pos>().unwrap();
    assert_eq!(store.get_component(0).unwrap().0, 1.0);
    assert_eq!(store.get_component(0).unwrap().1, 2.0);
    assert_eq!(store.get_component(1).unwrap().0, 9.0);
    assert_eq!(store.get_component(1).unwrap().1, 9.0);
}

#[test]
fn system_write_two() {
    let mut world = setup_world();
    let mut schedule = Schedule::new();
    schedule.add_system(double_both);
    schedule.run(&mut world);

    let pstore = world.get_component_store::<Pos>().unwrap();
    let vstore = world.get_component_store::<Vel>().unwrap();
    assert_eq!(pstore.get_component(0).unwrap().0, 0.0);
    assert_eq!(vstore.get_component(0).unwrap().0, 2.0);
    assert_eq!(pstore.get_component(1).unwrap().0, 20.0);
    assert_eq!(vstore.get_component(1).unwrap().0, -2.0);
}

#[test]
fn system_three_params() {
    let mut world = {
        let mut w = World::new();
        let e = w.spawn_entity();
        w.attach_component(&e, Pos(1.0, 2.0)).unwrap();
        w.attach_component(&e, Vel(3.0, 4.0)).unwrap();
        w.attach_component(&e, 100i32).unwrap();
        w
    };

    let mut schedule = Schedule::new();
    schedule.add_system(update_score);
    schedule.run(&mut world);

    let pstore = world.get_component_store::<Pos>().unwrap();
    let sstore = world.get_component_store::<i32>().unwrap();
    assert_eq!(pstore.get_component(0).unwrap().0, 4.0);
    assert_eq!(pstore.get_component(0).unwrap().1, 6.0);
    assert_eq!(*sstore.get_component(0).unwrap(), 101);
}

#[test]
fn system_four_params() {
    let mut world = {
        let mut w = World::new();
        let e = w.spawn_entity();
        w.attach_component(&e, Pos(1.0, 2.0)).unwrap();
        w.attach_component(&e, Vel(3.0, 4.0)).unwrap();
        w.attach_component(&e, 100i32).unwrap();
        w.attach_component(&e, true).unwrap();
        w
    };

    let mut schedule = Schedule::new();
    schedule.add_system(update_all);
    schedule.run(&mut world);

    let pstore = world.get_component_store::<Pos>().unwrap();
    assert_eq!(pstore.get_component(0).unwrap().0, 4.0);
}

#[test]
fn system_no_conflict_batching() {
    let mut world = setup_world();
    let call_order = Arc::new(Mutex::new(Vec::new()));

    let mut schedule = Schedule::new();
    {
        let co = Arc::clone(&call_order);
        schedule.add_system::<fn(&Pos), _>(move |_pos: &Pos| co.lock().unwrap().push("read_pos"));
    }
    {
        let co = Arc::clone(&call_order);
        schedule.add_system::<fn(&Vel), _>(move |_vel: &Vel| co.lock().unwrap().push("read_vel"));
    }
    schedule.run(&mut world);

    let order = call_order.lock().unwrap();
    assert!(order.contains(&"read_pos"));
    assert!(order.contains(&"read_vel"));
    assert_eq!(order.len(), 6);
}

#[test]
fn system_snapshot_isolation() {
    let mut world = {
        let mut w = World::new();
        let e1 = w.spawn_entity();
        w.attach_component(&e1, Pos(0.0, 0.0)).unwrap();
        let e2 = w.spawn_entity();
        w.attach_component(&e2, Pos(100.0, 100.0)).unwrap();
        w
    };

    let mut schedule = Schedule::new();
    schedule.add_system(add_five);
    schedule.add_system::<fn(&Pos), _>(|_pos: &Pos| {});
    schedule.run(&mut world);

    let store = world.get_component_store::<Pos>().unwrap();
    assert_eq!(store.get_component(0).unwrap().0, 5.0);
    assert_eq!(store.get_component(1).unwrap().0, 105.0);
}

#[test]
fn system_multiple_sequential_runs() {
    let mut world = setup_world();
    let mut schedule = Schedule::new();
    schedule.add_system(move_pos);

    schedule.run(&mut world);
    schedule.run(&mut world);
    schedule.run(&mut world);

    let store = world.get_component_store::<Pos>().unwrap();
    assert_eq!(store.get_component(0).unwrap().0, 3.0);
}

#[test]
fn system_empty_world() {
    let mut world = World::new();
    let mut schedule = Schedule::new();
    let called = Arc::new(Mutex::new(false));
    {
        let c = Arc::clone(&called);
        schedule.add_system::<fn(&mut Pos), _>(move |_: &mut Pos| *c.lock().unwrap() = true);
    }
    schedule.run(&mut world);
    assert!(!*called.lock().unwrap());
}

#[test]
fn system_closure_capture() {
    let mut world = setup_world();
    let multiplier = 10.0;

    let mut schedule = Schedule::new();
    schedule.add_system::<fn(&mut Pos), _>(move |pos: &mut Pos| pos.0 *= multiplier);
    schedule.run(&mut world);

    let store = world.get_component_store::<Pos>().unwrap();
    assert_eq!(store.get_component(0).unwrap().0, 0.0);
    assert_eq!(store.get_component(1).unwrap().0, 100.0);
}

#[test]
fn system_multiple_schedules_same_world() {
    let mut world = setup_world();

    let mut sched_a = Schedule::new();
    sched_a.add_system::<fn(&mut Pos), _>(|pos: &mut Pos| pos.0 += 1.0);
    sched_a.run(&mut world);

    let mut sched_b = Schedule::new();
    sched_b.add_system::<fn(&mut Pos), _>(|pos: &mut Pos| pos.0 += 10.0);
    sched_b.run(&mut world);

    let store = world.get_component_store::<Pos>().unwrap();
    assert_eq!(store.get_component(0).unwrap().0, 11.0);
}

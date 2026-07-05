use criterion::Criterion;
use orbital_ecs::{IntoSystem, Schedule, Snapshot};

use super::common::{Pos, Vel, setup_dense_world};

const N: usize = 10_000;

// ---------------------------------------------------------------------------
// System function definitions (fn pointers work with IntoSystem<M>)
// ---------------------------------------------------------------------------

fn add_one(pos: &mut Pos) {
    pos.0 += 1.0;
}

fn read_pos(_pos: &Pos) {}

fn read_pos_implies(_pos: &Pos) {}

fn apply_vel(pos: &mut Pos, vel: &Vel) {
    pos.0 += vel.0;
}

fn double_pos(pos: &mut Pos) {
    pos.0 *= 2.0;
}

fn double_vel(_vel: &mut Vel) {}

// ---------------------------------------------------------------------------
// Run benchmarks
// ---------------------------------------------------------------------------

pub fn bench_system_run_1w(c: &mut Criterion) {
    let world = setup_dense_world(N);
    let mut schedule = Schedule::new();
    schedule.add_system(add_one);

    c.bench_function("system_run_1w", |b| {
        b.iter(|| {
            schedule.run(&world);
        });
    });
}

pub fn bench_system_run_1r(c: &mut Criterion) {
    let world = setup_dense_world(N);
    let mut schedule = Schedule::new();
    schedule.add_system(read_pos);

    c.bench_function("system_run_1r", |b| {
        b.iter(|| {
            schedule.run(&world);
        });
    });
}

pub fn bench_system_run_2wr(c: &mut Criterion) {
    let world = setup_dense_world(N);
    let mut schedule = Schedule::new();
    schedule.add_system(apply_vel);

    c.bench_function("system_run_2wr", |b| {
        b.iter(|| {
            schedule.run(&world);
        });
    });
}

pub fn bench_system_batch_no_conflict(c: &mut Criterion) {
    let world = setup_dense_world(N);
    let mut schedule = Schedule::new();
    schedule.add_system(read_pos);
    schedule.add_system(read_pos_implies);

    c.bench_function("system_batch_no_conflict", |b| {
        b.iter(|| {
            schedule.run(&world);
        });
    });
}

pub fn bench_system_batch_conflict(c: &mut Criterion) {
    let world = setup_dense_world(N);
    let mut schedule = Schedule::new();
    schedule.add_system(double_pos);
    schedule.add_system(add_one);

    c.bench_function("system_batch_conflict", |b| {
        b.iter(|| {
            schedule.run(&world);
        });
    });
}

pub fn bench_system_run_many_sequential(c: &mut Criterion) {
    let world = setup_dense_world(N);
    let mut schedule = Schedule::new();
    for _ in 0..16 {
        schedule.add_system(double_pos);
    }

    c.bench_function("system_run_many_sequential", |b| {
        b.iter(|| {
            schedule.run(&world);
        });
    });
}

pub fn bench_system_run_many_parallel(c: &mut Criterion) {
    const K: usize = 16;
    let world = setup_dense_world(N);
    let mut schedule = Schedule::new();
    // Each system writes a different dummy store — no conflicts.
    for _ in 0..K {
        schedule.add_system(add_one);
        schedule.add_system(double_vel);
    }

    c.bench_function("system_run_many_parallel", |b| {
        b.iter(|| {
            schedule.run(&world);
        });
    });
}

pub fn bench_system_create(c: &mut Criterion) {
    c.bench_function("system_create", |b| {
        b.iter(|| {
            let sys = IntoSystem::<fn(&mut Pos)>::into_system(add_one);
            std::hint::black_box(sys);
        });
    });
}

pub fn bench_system_schedule_build(c: &mut Criterion) {
    let mut schedule = Schedule::new();
    for _ in 0..64 {
        schedule.add_system(add_one);
    }

    // Force re-building batches each iteration by cloning schedule.
    c.bench_function("system_schedule_build", |b| {
        b.iter(|| {
            let _batches = schedule.build_batches();
        });
    });
}

pub fn bench_system_clone_snapshot(c: &mut Criterion) {
    let world = setup_dense_world(N);
    let store = world
        .get_component_store::<Pos>()
        .expect("Pos store missing");

    c.bench_function("system_clone_snapshot", |b| {
        b.iter(|| {
            let snap = Snapshot::clone_from_store(&store);
            std::hint::black_box(snap);
        });
    });
}

pub fn bench_system_run_empty_world(c: &mut Criterion) {
    let world = super::common::setup_dense_world(0);
    let mut schedule = Schedule::new();
    schedule.add_system(add_one);

    c.bench_function("system_run_empty_world", |b| {
        b.iter(|| {
            schedule.run(&world);
        });
    });
}

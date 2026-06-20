use std::ops::Range;

use criterion::{Criterion, criterion_group, criterion_main};
use orbital_ecs::{Entity, Query, Read, With, Without, World, Write};
use rand::{RngExt, seq::SliceRandom};

// ---------------------------------------------------------------------------
// Component types for benchmarks
// ---------------------------------------------------------------------------

#[allow(dead_code)]
#[derive(Debug)]
struct Pos(f32, f32);
#[allow(dead_code)]
#[derive(Debug)]
struct Vel(f32, f32);
#[allow(dead_code)]
#[derive(Debug)]
struct Name(String);
#[allow(dead_code)]
#[derive(Debug)]
struct Score(i32);
#[allow(dead_code)]
#[derive(Debug)]
struct Frozen;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn setup_dense_world(n: usize) -> World {
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

fn setup_partial_world(n: usize) -> World {
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

// ---------------------------------------------------------------------------
// Entity / component benchmarks (existing)
// ---------------------------------------------------------------------------

fn bench_entity_spawn(c: &mut Criterion) {
    let mut world = World::new();
    c.bench_function("entity_spawn", |x| {
        x.iter(|| {
            world.spawn_entity();
        });
    });
}

fn bench_entity_spawn_despawn(c: &mut Criterion) {
    const RANGE: Range<i32> = 0..10_000;

    c.bench_function("entity_spawn_despawn", |x| {
        x.iter(|| {
            let mut world = World::new();
            let mut entities = Vec::new();
            for _ in RANGE {
                let entity = world.spawn_entity();
                entities.push(entity);
            }

            let mut rng = rand::rng();
            entities.shuffle(&mut rng);

            while let Some(entity) = entities.pop() {
                world.despawn_entity(&entity);
            }
        });
    });
}

fn bench_component_attach(c: &mut Criterion) {
    const RANGE: Range<i32> = 0..10_000;

    let mut world = World::new();
    let mut entities = Vec::new();
    for _ in RANGE {
        let entity = world.spawn_entity();
        entities.push(entity);
    }

    let mut rng = rand::rng();
    entities.shuffle(&mut rng);

    c.bench_function("component_attach", |x| {
        x.iter(|| {
            let entity_id = rng.random_range(RANGE) as usize;
            world
                .attach_component(&Entity::new(entity_id, 0), format!("Test #{entity_id}"))
                .unwrap_or_else(|_| panic!("Attachment failure at {entity_id}"));
        })
    });
}

// ---------------------------------------------------------------------------
// Query benchmarks
// ---------------------------------------------------------------------------

fn bench_query_read_1(c: &mut Criterion) {
    const N: usize = 10_000;
    let world = setup_dense_world(N);
    c.bench_function("query_read_1", |b| {
        b.iter(|| {
            let mut q: Query<(Read<Pos>,)> = Query::new(&world);
            std::hint::black_box(q.iter().count());
        });
    });
}

fn bench_query_read_2(c: &mut Criterion) {
    const N: usize = 10_000;
    let world = setup_dense_world(N);
    c.bench_function("query_read_2", |b| {
        b.iter(|| {
            let mut q: Query<(Read<Pos>, Read<Vel>)> = Query::new(&world);
            std::hint::black_box(q.iter().count());
        });
    });
}

fn bench_query_read_3(c: &mut Criterion) {
    const N: usize = 10_000;
    let world = setup_dense_world(N);
    c.bench_function("query_read_3", |b| {
        b.iter(|| {
            let mut q: Query<(Read<Pos>, Read<Vel>, Read<Name>)> = Query::new(&world);
            std::hint::black_box(q.iter().count());
        });
    });
}

fn bench_query_read_4(c: &mut Criterion) {
    const N: usize = 10_000;
    let world = setup_dense_world(N);
    c.bench_function("query_read_4", |b| {
        b.iter(|| {
            let mut q: Query<(Read<Pos>, Read<Vel>, Read<Name>, Read<Score>)> =
                Query::new(&world);
            std::hint::black_box(q.iter().count());
        });
    });
}

fn bench_query_write_1(c: &mut Criterion) {
    const N: usize = 10_000;
    let world = setup_dense_world(N);
    c.bench_function("query_write_1", |b| {
        b.iter(|| {
            let mut q: Query<(Write<Pos>,)> = Query::new(&world);
            for (pos,) in q.iter() {
                pos.0 += 1.0;
            }
        });
    });
}

fn bench_query_write_2(c: &mut Criterion) {
    const N: usize = 10_000;
    let world = setup_dense_world(N);
    c.bench_function("query_write_2", |b| {
        b.iter(|| {
            let mut q: Query<(Write<Pos>, Read<Vel>)> = Query::new(&world);
            for (pos, vel) in q.iter() {
                pos.0 += vel.0;
            }
        });
    });
}

fn bench_query_write_3(c: &mut Criterion) {
    const N: usize = 10_000;
    let world = setup_dense_world(N);
    c.bench_function("query_write_3", |b| {
        b.iter(|| {
            let mut q: Query<(Write<Pos>, Read<Vel>, Read<Name>)> = Query::new(&world);
            for (pos, vel, _name) in q.iter() {
                pos.0 += vel.0;
            }
        });
    });
}

fn bench_query_write_4(c: &mut Criterion) {
    const N: usize = 10_000;
    let world = setup_dense_world(N);
    c.bench_function("query_write_4", |b| {
        b.iter(|| {
            let mut q: Query<(Write<Pos>, Read<Vel>, Read<Name>, Read<Score>)> =
                Query::new(&world);
            for (pos, vel, _name, _score) in q.iter() {
                pos.0 += vel.0;
            }
        });
    });
}

fn bench_query_create_1(c: &mut Criterion) {
    const N: usize = 10_000;
    let world = setup_dense_world(N);
    c.bench_function("query_create_1", |b| {
        b.iter(|| {
            let mut q: Query<(Read<Pos>,)> = Query::new(&world);
            std::hint::black_box(q.iter().count());
        });
    });
}

fn bench_query_create_2(c: &mut Criterion) {
    const N: usize = 10_000;
    let world = setup_dense_world(N);
    c.bench_function("query_create_2", |b| {
        b.iter(|| {
            let mut q: Query<(Read<Pos>, Read<Vel>)> = Query::new(&world);
            std::hint::black_box(q.iter().count());
        });
    });
}

fn bench_query_create_3(c: &mut Criterion) {
    const N: usize = 10_000;
    let world = setup_dense_world(N);
    c.bench_function("query_create_3", |b| {
        b.iter(|| {
            let mut q: Query<(Read<Pos>, Read<Vel>, Read<Name>)> = Query::new(&world);
            std::hint::black_box(q.iter().count());
        });
    });
}

fn bench_query_create_4(c: &mut Criterion) {
    const N: usize = 10_000;
    let world = setup_dense_world(N);
    c.bench_function("query_create_4", |b| {
        b.iter(|| {
            let mut q: Query<(Read<Pos>, Read<Vel>, Read<Name>, Read<Score>)> =
                Query::new(&world);
            std::hint::black_box(q.iter().count());
        });
    });
}

fn bench_query_filter_with_1(c: &mut Criterion) {
    const N: usize = 10_000;
    let world = setup_dense_world(N);
    c.bench_function("query_filter_with_1", |b| {
        b.iter(|| {
            let mut q: Query<(Read<Pos>,), With<Name>> = Query::new(&world);
            std::hint::black_box(q.iter().count());
        });
    });
}

fn bench_query_filter_without_1(c: &mut Criterion) {
    const N: usize = 10_000;
    let world = setup_dense_world(N);
    c.bench_function("query_filter_without_1", |b| {
        b.iter(|| {
            let mut q: Query<(Read<Pos>,), Without<Frozen>> = Query::new(&world);
            std::hint::black_box(q.iter().count());
        });
    });
}

fn bench_query_filter_with_2(c: &mut Criterion) {
    const N: usize = 10_000;
    let world = setup_dense_world(N);
    c.bench_function("query_filter_with_2", |b| {
        b.iter(|| {
            let mut q: Query<(Read<Pos>,), (With<Name>, With<Score>)> = Query::new(&world);
            std::hint::black_box(q.iter().count());
        });
    });
}

fn bench_query_filter_with_3(c: &mut Criterion) {
    const N: usize = 10_000;
    let world = setup_dense_world(N);
    c.bench_function("query_filter_with_3", |b| {
        b.iter(|| {
            let mut q: Query<(Read<Pos>,), (With<Name>, With<Score>, With<Vel>)> =
                Query::new(&world);
            std::hint::black_box(q.iter().count());
        });
    });
}

fn bench_query_partial_match(c: &mut Criterion) {
    const N: usize = 10_000;
    let world = setup_partial_world(N);
    c.bench_function("query_partial_match", |b| {
        b.iter(|| {
            let mut q: Query<(Read<Pos>, Read<Vel>)> = Query::new(&world);
            std::hint::black_box(q.iter().count());
        });
    });
}

criterion_group!(
    benches_entity,
    bench_entity_spawn,
    bench_entity_spawn_despawn
);
criterion_group!(benches_component, bench_component_attach);
criterion_group!(
    benches_query,
    bench_query_read_1,
    bench_query_read_2,
    bench_query_read_3,
    bench_query_read_4,
    bench_query_write_1,
    bench_query_write_2,
    bench_query_write_3,
    bench_query_write_4,
    bench_query_create_1,
    bench_query_create_2,
    bench_query_create_3,
    bench_query_create_4,
    bench_query_filter_with_1,
    bench_query_filter_without_1,
    bench_query_filter_with_2,
    bench_query_filter_with_3,
    bench_query_partial_match,
);
criterion_main!(benches_entity, benches_component, benches_query);

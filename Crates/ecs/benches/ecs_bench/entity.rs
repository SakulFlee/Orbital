use std::ops::Range;

use criterion::Criterion;
use orbital_ecs::World;
use rand::seq::SliceRandom;

use super::common::setup_dense_world;

const N: usize = 10_000;
const RANGE: Range<i32> = 0..10_000;

pub fn bench_entity_spawn(c: &mut Criterion) {
    let mut world = World::new();
    c.bench_function("entity_spawn", |x| {
        x.iter(|| {
            world.spawn_entity();
        });
    });
}

pub fn bench_entity_spawn_despawn(c: &mut Criterion) {
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

pub fn bench_world_spawn_bulk(c: &mut Criterion) {
    c.bench_function("world_spawn_bulk", |b| {
        b.iter(|| {
            let mut world = World::new();
            for _ in 0..N {
                world.spawn_entity();
            }
            std::hint::black_box(world);
        });
    });
}

pub fn bench_world_despawn_bulk(c: &mut Criterion) {
    c.bench_function("world_despawn_bulk", |b| {
        b.iter(|| {
            let mut world = World::new();
            let entities: Vec<_> = (0..N).map(|_| world.spawn_entity()).collect();
            for e in &entities {
                world.despawn_entity(e);
            }
        });
    });
}

pub fn bench_world_get_store_read(c: &mut Criterion) {
    let world = setup_dense_world(N);
    c.bench_function("world_get_store_read", |b| {
        b.iter(|| {
            let store = world.get_component_store::<super::common::Pos>();
            std::hint::black_box(store);
        });
    });
}

pub fn bench_world_get_store_write(c: &mut Criterion) {
    let world = setup_dense_world(N);
    c.bench_function("world_get_store_write", |b| {
        b.iter(|| {
            let store = world.get_component_store_mut::<super::common::Pos>();
            std::hint::black_box(store);
        });
    });
}

pub fn bench_world_attach_detach_cycle(c: &mut Criterion) {
    let mut world = World::new();
    let entities: Vec<_> = (0..N).map(|_| world.spawn_entity()).collect();
    let mut i = 0usize;

    c.bench_function("world_attach_detach_cycle", |b| {
        b.iter(|| {
            let e = &entities[i % N];
            world.attach_component(e, 42i32).unwrap();
            world.detach_component::<i32>(e).unwrap();
            i += 1;
        });
    });
}

use std::ops::Range;

use criterion::{Criterion, criterion_group, criterion_main};
use orbital_ecs::{Entity, World};
use rand::{RngExt, seq::SliceRandom};

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

criterion_group!(
    benches_entity,
    bench_entity_spawn,
    bench_entity_spawn_despawn
);
criterion_group!(benches_component, bench_component_attach);
criterion_main!(benches_entity, benches_component);

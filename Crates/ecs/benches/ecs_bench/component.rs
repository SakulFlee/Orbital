use std::ops::Range;

use criterion::Criterion;
use orbital_ecs::Entity;
use rand::prelude::SliceRandom;
use rand::RngExt;

use super::common::setup_dense_world;

const RANGE: Range<i32> = 0..10_000;
const N: usize = 10_000;

pub fn bench_component_attach(c: &mut Criterion) {
    let mut world = super::common::setup_dense_world(0);
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

pub fn bench_component_detach(c: &mut Criterion) {
    c.bench_function("component_detach", |b| {
        b.iter(|| {
            let mut w = super::common::setup_dense_world(0);
            let es: Vec<_> = (0..N)
                .map(|i| {
                    let e = w.spawn_entity();
                    w.attach_component(&e, i as f64).unwrap();
                    e
                })
                .collect();
            for e in &es {
                w.detach_component::<f64>(e).unwrap();
            }
        });
    });
}

pub fn bench_component_get(c: &mut Criterion) {
    let world = setup_dense_world(N);
    let store_handle = world
        .get_component_store::<super::common::Pos>()
        .expect("Pos store missing");

    c.bench_function("component_get", |b| {
        b.iter(|| {
            for &eid in store_handle.dense.as_slice() {
                if let Some(comp) = store_handle.get_component(eid) {
                    std::hint::black_box(comp);
                }
            }
        });
    });
}

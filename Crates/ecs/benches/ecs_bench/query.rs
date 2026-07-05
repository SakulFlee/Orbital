use criterion::Criterion;
use orbital_ecs::{Query, Read, With, Without, Write};

use super::common::{Frozen, Name, Pos, Score, Vel, setup_dense_world, setup_partial_world};

const N: usize = 10_000;

// ---------------------------------------------------------------------------
// Query iteration (read)
// ---------------------------------------------------------------------------

pub fn bench_query_read_1(c: &mut Criterion) {
    let world = setup_dense_world(N);
    c.bench_function("query_read_1", |b| {
        b.iter(|| {
            let mut q: Query<(Read<Pos>,)> = Query::new(&world);
            std::hint::black_box(q.iter().count());
        });
    });
}

pub fn bench_query_read_2(c: &mut Criterion) {
    let world = setup_dense_world(N);
    c.bench_function("query_read_2", |b| {
        b.iter(|| {
            let mut q: Query<(Read<Pos>, Read<Vel>)> = Query::new(&world);
            std::hint::black_box(q.iter().count());
        });
    });
}

pub fn bench_query_read_3(c: &mut Criterion) {
    let world = setup_dense_world(N);
    c.bench_function("query_read_3", |b| {
        b.iter(|| {
            let mut q: Query<(Read<Pos>, Read<Vel>, Read<Name>)> = Query::new(&world);
            std::hint::black_box(q.iter().count());
        });
    });
}

pub fn bench_query_read_4(c: &mut Criterion) {
    let world = setup_dense_world(N);
    c.bench_function("query_read_4", |b| {
        b.iter(|| {
            let mut q: Query<(Read<Pos>, Read<Vel>, Read<Name>, Read<Score>)> = Query::new(&world);
            std::hint::black_box(q.iter().count());
        });
    });
}

// ---------------------------------------------------------------------------
// Query iteration (write)
// ---------------------------------------------------------------------------

pub fn bench_query_write_1(c: &mut Criterion) {
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

pub fn bench_query_write_2(c: &mut Criterion) {
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

pub fn bench_query_write_3(c: &mut Criterion) {
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

pub fn bench_query_write_4(c: &mut Criterion) {
    let world = setup_dense_world(N);
    c.bench_function("query_write_4", |b| {
        b.iter(|| {
            let mut q: Query<(Write<Pos>, Read<Vel>, Read<Name>, Read<Score>)> = Query::new(&world);
            for (pos, vel, _name, _score) in q.iter() {
                pos.0 += vel.0;
            }
        });
    });
}

// ---------------------------------------------------------------------------
// Query creation overhead (identical to read_1–4 but measure creation + count)
// ---------------------------------------------------------------------------

pub fn bench_query_create_1(c: &mut Criterion) {
    let world = setup_dense_world(N);
    c.bench_function("query_create_1", |b| {
        b.iter(|| {
            let mut q: Query<(Read<Pos>,)> = Query::new(&world);
            std::hint::black_box(q.iter().count());
        });
    });
}

pub fn bench_query_create_2(c: &mut Criterion) {
    let world = setup_dense_world(N);
    c.bench_function("query_create_2", |b| {
        b.iter(|| {
            let mut q: Query<(Read<Pos>, Read<Vel>)> = Query::new(&world);
            std::hint::black_box(q.iter().count());
        });
    });
}

pub fn bench_query_create_3(c: &mut Criterion) {
    let world = setup_dense_world(N);
    c.bench_function("query_create_3", |b| {
        b.iter(|| {
            let mut q: Query<(Read<Pos>, Read<Vel>, Read<Name>)> = Query::new(&world);
            std::hint::black_box(q.iter().count());
        });
    });
}

pub fn bench_query_create_4(c: &mut Criterion) {
    let world = setup_dense_world(N);
    c.bench_function("query_create_4", |b| {
        b.iter(|| {
            let mut q: Query<(Read<Pos>, Read<Vel>, Read<Name>, Read<Score>)> = Query::new(&world);
            std::hint::black_box(q.iter().count());
        });
    });
}

// ---------------------------------------------------------------------------
// Query filters
// ---------------------------------------------------------------------------

pub fn bench_query_filter_with_1(c: &mut Criterion) {
    let world = setup_dense_world(N);
    c.bench_function("query_filter_with_1", |b| {
        b.iter(|| {
            let mut q: Query<(Read<Pos>,), With<Name>> = Query::new(&world);
            std::hint::black_box(q.iter().count());
        });
    });
}

pub fn bench_query_filter_without_1(c: &mut Criterion) {
    let world = setup_dense_world(N);
    c.bench_function("query_filter_without_1", |b| {
        b.iter(|| {
            let mut q: Query<(Read<Pos>,), Without<Frozen>> = Query::new(&world);
            std::hint::black_box(q.iter().count());
        });
    });
}

pub fn bench_query_filter_with_2(c: &mut Criterion) {
    let world = setup_dense_world(N);
    c.bench_function("query_filter_with_2", |b| {
        b.iter(|| {
            let mut q: Query<(Read<Pos>,), (With<Name>, With<Score>)> = Query::new(&world);
            std::hint::black_box(q.iter().count());
        });
    });
}

pub fn bench_query_filter_with_3(c: &mut Criterion) {
    let world = setup_dense_world(N);
    c.bench_function("query_filter_with_3", |b| {
        b.iter(|| {
            let mut q: Query<(Read<Pos>,), (With<Name>, With<Score>, With<Vel>)> =
                Query::new(&world);
            std::hint::black_box(q.iter().count());
        });
    });
}

pub fn bench_query_partial_match(c: &mut Criterion) {
    let world = setup_partial_world(N);
    c.bench_function("query_partial_match", |b| {
        b.iter(|| {
            let mut q: Query<(Read<Pos>, Read<Vel>)> = Query::new(&world);
            std::hint::black_box(q.iter().count());
        });
    });
}

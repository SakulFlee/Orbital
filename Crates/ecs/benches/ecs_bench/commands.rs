use criterion::Criterion;
use orbital_ecs::Commands;

use super::common::{setup_dense_world, Pos};

const N: usize = 10_000;

pub fn bench_commands_spawn(c: &mut Criterion) {
    let mut world = setup_dense_world(0);
    c.bench_function("commands_spawn", |b| {
        b.iter(|| {
            let mut cmds = Commands::new();
            for _ in 0..N {
                cmds.spawn_entity();
            }
            cmds.flush(&mut world).unwrap();
        });
    });
}

pub fn bench_commands_attach(c: &mut Criterion) {
    let mut world = setup_dense_world(N);
    c.bench_function("commands_attach", |b| {
        b.iter(|| {
            let mut cmds = Commands::new();
            for i in 0..N {
                let e = world.spawn_entity();
                cmds.attach_component(&e, Pos(i as f32, 0.0));
            }
            cmds.flush(&mut world).unwrap();
        });
    });
}

pub fn bench_commands_mixed(c: &mut Criterion) {
    let mut world = setup_dense_world(0);
    c.bench_function("commands_mixed", |b| {
        b.iter(|| {
            let mut cmds = Commands::new();
            for i in 0..(N / 2) {
                let e = cmds.spawn_entity();
                cmds.attach_component(&e, Pos(i as f32, 0.0));
            }
            cmds.flush(&mut world).unwrap();
        });
    });
}

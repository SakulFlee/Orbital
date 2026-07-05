#[path = "ecs_bench/commands.rs"]
mod commands;
#[path = "ecs_bench/common.rs"]
mod common;
#[path = "ecs_bench/component.rs"]
mod component;
#[path = "ecs_bench/entity.rs"]
mod entity;
#[path = "ecs_bench/query.rs"]
mod query;
#[path = "ecs_bench/system.rs"]
mod system;

use criterion::{criterion_group, criterion_main};

criterion_group!(
    benches_entity,
    entity::bench_entity_spawn,
    entity::bench_entity_spawn_despawn,
    entity::bench_world_spawn_bulk,
    entity::bench_world_despawn_bulk,
    entity::bench_world_get_store_read,
    entity::bench_world_get_store_write,
    entity::bench_world_attach_detach_cycle,
);

criterion_group!(
    benches_component,
    component::bench_component_attach,
    component::bench_component_detach,
    component::bench_component_get,
);

criterion_group!(
    benches_query,
    query::bench_query_read_1,
    query::bench_query_read_2,
    query::bench_query_read_3,
    query::bench_query_read_4,
    query::bench_query_write_1,
    query::bench_query_write_2,
    query::bench_query_write_3,
    query::bench_query_write_4,
    query::bench_query_create_1,
    query::bench_query_create_2,
    query::bench_query_create_3,
    query::bench_query_create_4,
    query::bench_query_filter_with_1,
    query::bench_query_filter_without_1,
    query::bench_query_filter_with_2,
    query::bench_query_filter_with_3,
    query::bench_query_partial_match,
);

criterion_group!(
    benches_system,
    system::bench_system_run_1w,
    system::bench_system_run_1r,
    system::bench_system_run_2wr,
    system::bench_system_batch_no_conflict,
    system::bench_system_batch_conflict,
    system::bench_system_run_many_sequential,
    system::bench_system_run_many_parallel,
    system::bench_system_create,
    system::bench_system_schedule_build,
    system::bench_system_clone_snapshot,
    system::bench_system_run_empty_world,
);

criterion_group!(
    benches_commands,
    commands::bench_commands_spawn,
    commands::bench_commands_attach,
    commands::bench_commands_mixed,
);

criterion_main!(
    benches_entity,
    benches_component,
    benches_query,
    benches_system,
    benches_commands,
);

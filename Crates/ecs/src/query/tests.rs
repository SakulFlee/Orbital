use crate::World;
use crate::query::*;

#[derive(Debug)]
struct Pos(f32, f32);
#[derive(Debug)]
struct Vel(f32, f32);
#[derive(Debug)]
struct Frozen;
#[allow(dead_code)]
#[derive(Debug)]
struct Name(String);
#[allow(dead_code)]
#[derive(Debug)]
struct Score(i32);

fn setup_world() -> World {
    let mut world = World::new();

    let e1 = world.spawn_entity();
    world.attach_component(&e1, Pos(0.0, 0.0)).unwrap();
    world.attach_component(&e1, Vel(1.0, 0.0)).unwrap();
    world.attach_component(&e1, Name("moving".into())).unwrap();

    let e2 = world.spawn_entity();
    world.attach_component(&e2, Pos(10.0, 10.0)).unwrap();
    world.attach_component(&e2, Vel(0.0, -1.0)).unwrap();

    let e3 = world.spawn_entity();
    world.attach_component(&e3, Pos(5.0, 5.0)).unwrap();
    world.attach_component(&e3, Frozen).unwrap();

    world
}

#[test]
fn query_basic() {
    let world = setup_world();
    let mut query: Query<(Read<Pos>, Read<Vel>)> = Query::new(&world);
    let mut count = 0;
    for (_pos, _vel) in query.iter() {
        count += 1;
    }
    assert_eq!(count, 2, "Only e1 and e2 have both Pos and Vel");
}

#[test]
fn query_write() {
    let world = setup_world();
    {
        let mut query: Query<(Write<Pos>, Read<Vel>)> = Query::new(&world);
        for (pos, vel) in query.iter() {
            pos.0 += vel.0;
            pos.1 += vel.1;
        }
    }

    let store = world.get_component_store::<Pos>().unwrap();
    let p1 = store.get_component(0).unwrap();
    assert_eq!(p1.0, 1.0);
    assert_eq!(p1.1, 0.0);
}

#[test]
fn query_with_filter() {
    let world = setup_world();
    let mut query: Query<(Read<Pos>,), With<Name>> = Query::new(&world);
    let count = query.iter().count();
    assert_eq!(count, 1, "Only e1 has both Pos and Name");
}

#[test]
fn query_without_filter() {
    let world = setup_world();
    let mut query: Query<(Read<Pos>,), Without<Frozen>> = Query::new(&world);
    let count = query.iter().count();
    assert_eq!(count, 2, "e1 and e2 have Pos without Frozen");
}

#[test]
fn query_combined_filter() {
    let world = setup_world();
    let mut query: Query<(Read<Pos>, Read<Vel>), (With<Name>, Without<Frozen>)> =
        Query::new(&world);
    let count = query.iter().count();
    assert_eq!(count, 1, "Only e1 has Pos, Vel, Name, and not Frozen");
}

#[test]
fn query_no_match() {
    let world = setup_world();
    let mut query: Query<(Read<Vel>,), Without<Vel>> = Query::new(&world);
    let count = query.iter().count();
    assert_eq!(
        count, 0,
        "No entity can simultaneously have and not have Vel"
    );
}

#[test]
fn query_single_read() {
    let world = setup_world();
    let mut query: Query<(Read<Pos>,)> = Query::new(&world);
    let count = query.iter().count();
    assert_eq!(count, 3, "All 3 entities have Pos");
}

#[test]
fn query_single_write() {
    let world = setup_world();
    {
        let mut query: Query<(Write<Pos>,)> = Query::new(&world);
        for (pos,) in query.iter() {
            pos.0 *= 2.0;
        }
    }
    let store = world.get_component_store::<Pos>().unwrap();
    assert_eq!(store.get_component(0).unwrap().0, 0.0);
    assert_eq!(store.get_component(1).unwrap().0, 20.0);
    assert_eq!(store.get_component(2).unwrap().0, 10.0);
}

#[test]
fn query_three_components() {
    let world = setup_world();
    let mut query: Query<(Read<Pos>, Read<Vel>, Read<Name>)> = Query::new(&world);
    let count = query.iter().count();
    assert_eq!(count, 1, "Only e1 has all three");
}

#[test]
fn query_four_components() {
    let mut world = World::new();
    let e1 = world.spawn_entity();
    world.attach_component(&e1, Pos(1.0, 2.0)).unwrap();
    world.attach_component(&e1, Vel(3.0, 4.0)).unwrap();
    world.attach_component(&e1, Name("all".into())).unwrap();
    world.attach_component(&e1, Score(10)).unwrap();

    let e2 = world.spawn_entity();
    world.attach_component(&e2, Pos(5.0, 6.0)).unwrap();
    world.attach_component(&e2, Vel(7.0, 8.0)).unwrap();

    let mut query: Query<(Read<Pos>, Read<Vel>, Read<Name>, Read<Score>)> = Query::new(&world);
    let count = query.iter().count();
    assert_eq!(count, 1, "Only e1 has all four components");
}

#[test]
fn query_empty_world() {
    let mut world = World::new();
    let e = world.spawn_entity();
    world.attach_component(&e, Pos(0.0, 0.0)).unwrap();
    world.despawn_entity(&e);
    let mut query: Query<(Read<Pos>,)> = Query::new(&world);
    assert_eq!(query.iter().count(), 0);
}

#[test]
fn query_component_missing_from_some() {
    let mut world = World::new();
    let mut vel_entities = Vec::new();

    for i in 0..10_000 {
        let e = world.spawn_entity();
        world.attach_component(&e, Pos(i as f32, 0.0)).unwrap();
        if i % 2 == 0 {
            world.attach_component(&e, Vel(1.0, 0.0)).unwrap();
            vel_entities.push(e);
        }
    }

    let mut query: Query<(Read<Pos>, Read<Vel>)> = Query::new(&world);
    let results: Vec<_> = query.iter().collect();
    assert_eq!(
        results.len(),
        vel_entities.len(),
        "Should match entities with Vel"
    );
}

#[test]
fn query_write_then_read_scoped() {
    let mut world = World::new();
    let e = world.spawn_entity();
    world.attach_component(&e, Pos(100.0, 200.0)).unwrap();

    {
        let mut query: Query<(Write<Pos>,)> = Query::new(&world);
        for (pos,) in query.iter() {
            pos.0 = 999.0;
        }
    }

    let store = world.get_component_store::<Pos>().unwrap();
    assert_eq!(store.get_component(0).unwrap().0, 999.0);
}

#[test]
fn query_filter_without_all() {
    let mut world = World::new();
    let e1 = world.spawn_entity();
    world.attach_component(&e1, Pos(0.0, 0.0)).unwrap();
    let e2 = world.spawn_entity();
    world.attach_component(&e2, Pos(1.0, 1.0)).unwrap();

    let mut query: Query<(Read<Pos>,), Without<Frozen>> = Query::new(&world);
    assert_eq!(query.iter().count(), 2, "All have Pos, none have Frozen");
}

#[test]
fn query_filter_without_nonexistent() {
    let mut world = World::new();
    let e1 = world.spawn_entity();
    world.attach_component(&e1, Pos(0.0, 0.0)).unwrap();

    let mut query: Query<(Read<Pos>,), Without<Score>> = Query::new(&world);
    assert_eq!(query.iter().count(), 1, "Score type doesn't exist in world");
}

#[test]
fn query_filter_with_non_existent() {
    let mut world = World::new();
    let e1 = world.spawn_entity();
    world.attach_component(&e1, Pos(0.0, 0.0)).unwrap();

    let mut query: Query<(Read<Pos>,), With<Score>> = Query::new(&world);
    assert_eq!(query.iter().count(), 0, "No entity has Score");
}

#[test]
fn query_filter_four_tuple() {
    let mut world = World::new();
    let e1 = world.spawn_entity();
    world.attach_component(&e1, Pos(1.0, 1.0)).unwrap();
    world.attach_component(&e1, Vel(2.0, 2.0)).unwrap();
    world.attach_component(&e1, Name("a".into())).unwrap();

    let e2 = world.spawn_entity();
    world.attach_component(&e2, Pos(3.0, 3.0)).unwrap();
    world.attach_component(&e2, Vel(4.0, 4.0)).unwrap();

    let e3 = world.spawn_entity();
    world.attach_component(&e3, Pos(5.0, 5.0)).unwrap();

    let mut query: Query<
        (Read<Pos>, Read<Vel>),
        (With<Vel>, Without<Name>, Without<Frozen>, Without<Score>),
    > = Query::new(&world);
    assert_eq!(
        query.iter().count(),
        1,
        "Only e2 matches all filter conditions"
    );
}

#[test]
fn query_drop_mid_iteration() {
    let world = setup_world();
    let mut query: Query<(Read<Pos>, Read<Vel>)> = Query::new(&world);

    for item in query.iter().take(1) {
        let (_pos, _vel) = item;
    }
    drop(query);

    let store = world.get_component_store::<Pos>().unwrap();
    assert_eq!(store.get_component(0).unwrap().0, 0.0);
}

#[test]
fn query_entity_component_detach_affects_query() {
    let mut world = World::new();
    let e1 = world.spawn_entity();
    world.attach_component(&e1, Pos(1.0, 1.0)).unwrap();
    world.attach_component(&e1, Vel(2.0, 2.0)).unwrap();

    let e2 = world.spawn_entity();
    world.attach_component(&e2, Pos(3.0, 3.0)).unwrap();
    world.attach_component(&e2, Vel(4.0, 4.0)).unwrap();

    world.detach_component::<Vel>(&e2).unwrap();

    let mut query: Query<(Read<Pos>, Read<Vel>)> = Query::new(&world);
    assert_eq!(query.iter().count(), 1, "Only e1 still has Vel");
}

#[test]
fn query_store_non_contiguous_ids() {
    let mut world = World::new();
    let mut entities = Vec::new();

    for _ in 0..15 {
        let e = world.spawn_entity();
        entities.push(e);
        world.attach_component(&e, Pos(0.0, 0.0)).unwrap();
    }

    world.despawn_entity(&entities[5]);
    world.despawn_entity(&entities[7]);
    world.despawn_entity(&entities[10]);

    let mut query: Query<(Read<Pos>,)> = Query::new(&world);
    assert_eq!(query.iter().count(), 12, "3 entities despawned, 12 remain");
}

#[test]
fn query_multiple_simultaneous_reads() {
    let world = setup_world();

    let mut q1: Query<(Read<Pos>, Read<Vel>)> = Query::new(&world);
    let mut q2: Query<(Read<Pos>, Read<Name>)> = Query::new(&world);

    let count1 = q1.iter().count();
    let count2 = q2.iter().count();

    assert_eq!(count1, 2);
    assert_eq!(count2, 1);
}

mod entity;
pub use entity::*;

mod component;
pub use component::*;

mod world;
pub use world::*;

mod error;
pub use error::*;

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use rand::seq::SliceRandom;

    use crate::World;

    #[test]
    fn performance_validation_spawning_empty_entities() {
        const ENTITY_COUNT: usize = 1024 * 128;

        let mut world = World::new();
        let mut entities = Vec::with_capacity(ENTITY_COUNT);

        let earlier = Instant::now();

        for _ in 0..=ENTITY_COUNT {
            let entity = world.spawn_entity();
            entities.push(entity);
        }

        let duration = Instant::now().duration_since(earlier);
        println!("Duration: {:?}", duration);

        if duration.as_micros() >= 10_000 {
            panic!("Test took too long!");
        }
    }

    #[test]
    fn performance_validation_despawning_empty_entities() {
        const ENTITY_COUNT: usize = 1024 * 128;

        let mut world = World::new();
        let mut entities = Vec::with_capacity(ENTITY_COUNT);

        for _ in 0..=ENTITY_COUNT {
            let entity = world.spawn_entity();
            entities.push(entity);
        }

        // Shuffle entities around to make removing entities harder
        let mut rng = rand::rng();
        entities.shuffle(&mut rng);

        let earlier = Instant::now();

        entities.iter().for_each(|x| world.despawn_entity(x));

        let duration = Instant::now().duration_since(earlier);
        println!("Duration: {:?}", duration);

        if duration.as_secs() >= 7 {
            panic!("Test took too long!");
        }
    }

    #[test]
    fn performance_validation_spawning_one_type() {
        const ENTITY_COUNT: usize = 1024 * 128;

        let mut world = World::new();
        let mut entities = Vec::with_capacity(ENTITY_COUNT);

        for _ in 0..=ENTITY_COUNT {
            let entity = world.spawn_entity();
            entities.push(entity);
        }

        let mut rng = rand::rng();
        entities.shuffle(&mut rng);

        let earlier = Instant::now();
        entities.iter().enumerate().for_each(|(idx, entity_id)| {
            world
                .attach_component(entity_id, idx)
                .expect("Attachment failure")
        });

        let duration = Instant::now().duration_since(earlier);
        println!("Duration:\t{:?}", duration);

        if duration.as_micros() >= 10_000 {
            panic!("Test took too long!");
        }
    }
}

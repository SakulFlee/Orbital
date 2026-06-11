use ecs::{World, Component};

// Simple component examples - no need to manually implement Component
// due to blanket impl in component/mod.rs for all Any+Debug types

#[derive(Debug, Clone, Copy)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, Copy)]
struct Velocity {
    vx: f32,
    vy: f32,
}

fn main() {
    println!("=== ECS Simple Example ===\n");
    
    // Create world
    let mut world = World::new();
    println!("Created world");
    
    // Create an entity
    let entity = world.spawn_entity();
    println!("Spawned entity: {}", entity.index);
    
    // Attach a Position component (this is a placeholder call; actual storage not yet implemented)
    let pos = Position { x: 0.0, y: 0.0 };
    world.attach_component(&entity, pos);
    println!("Attached Position component");
    
    // Attempt to get the component (will return None until storage is implemented)
    if let Some(retrieved_pos) = world.get_component::<Position>(&entity) {
        println!("Retrieved Position: {:?}", retrieved_pos);
    } else {
        println!("Position component not yet retrievable (storage implementation pending)");
    }
    
    // Create another entity with Velocity
    let entity2 = world.spawn_entity();
    let vel = Velocity { vx: 1.0, vy: 2.0 };
    world.attach_component(&entity2, vel);
    println!("Spawned second entity and attached Velocity: {}", entity2.index);
    
    // Check validity
    println!("\nEntity validity checks:");
    println!("  Entity 1 valid: {}", world.is_valid(&entity));
    println!("  Entity 2 valid: {}", world.is_valid(&entity2));
    
    // Clean up
    world.despawn_entity(&entity);
    world.despawn_entity(&entity2);
    println!("\nDespawned both entities");
    println!("  Entity 1 valid: {}", world.is_valid(&entity));
    println!("  Entity 2 valid: {}", world.is_valid(&entity2));
    
    println!("\n=== Example completed ===");
}
use ecs::{World, Component};

// Simple component examples
#[derive(Debug, Clone, Copy)]
struct Position {
    x: f32,
    y: f32,
}

impl Component for Position {}

#[derive(Debug, Clone, Copy)]
struct Velocity {
    vx: f32,
    vy: f32,
}

impl Component for Velocity {}

#[derive(Debug, Clone)]
struct Health {
    current: i32,
    max: i32,
}

impl Component for Health {}

fn main() {
    println!("=== ECS Simple Example ===\n");
    
    // Create world
    let mut world = World::new();
    println!("Created world");
    
    // Create entities
    let player = world.spawn_entity();
    let enemy = world.spawn_entity();
    let projectile = world.spawn_entity();
    
    println!("Created 3 entities:");
    println!("  Player: {}", player.index);
    println!("  Enemy: {}", enemy.index);
    println!("  Projectile: {}", projectile.index);
    
    // Check validity
    println!("\nEntity validity checks:");
    println!("  Player valid: {}", world.is_valid(&player));
    println!("  Enemy valid: {}", world.is_valid(&enemy));
    println!("  Projectile valid: {}", world.is_valid(&projectile));
    
    // Clean up
    world.despawn_entity(&enemy);
    println!("\nDespawned enemy");
    println!("  Enemy valid: {}", world.is_valid(&enemy));
    
    // Final count
    println!("\nEntities remaining: 2 (player and projectile)");
    
    println!("\n=== Example completed successfully! ===");
}
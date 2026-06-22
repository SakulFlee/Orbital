/// Game-like movement simulation with multiple system stages.
///
/// Run: cargo run --example movement --release
use orbital_ecs::{Schedule, World};

#[derive(Debug, Clone)]
struct Pos(f32, f32);

#[derive(Debug, Clone)]
struct Vel(f32, f32);

#[derive(Debug, Clone)]
struct Gravity(f32);

fn apply_gravity(vel: &mut Vel, gravity: &Gravity) {
    vel.1 += gravity.0;
}

fn integrate(pos: &mut Pos, vel: &Vel) {
    pos.0 += vel.0;
    pos.1 += vel.1;
}

fn dampen(vel: &mut Vel) {
    vel.0 *= 0.99;
    vel.1 *= 0.99;
}

fn main() {
    let mut world = World::new();
    let e1 = world.spawn_entity();
    world.attach_component(&e1, Pos(0.0, 0.0)).unwrap();
    world.attach_component(&e1, Vel(10.0, 20.0)).unwrap();
    world.attach_component(&e1, Gravity(-9.8)).unwrap();

    let mut physics_a = Schedule::new();
    physics_a.add_system(apply_gravity);
    physics_a.add_system(dampen);

    let mut physics_b = Schedule::new();
    physics_b.add_system(integrate);

    for frame in 0..10 {
        physics_a.run(&world);
        physics_b.run(&world);

        let pos_store = world.get_component_store::<Pos>().unwrap();
        let vel_store = world.get_component_store::<Vel>().unwrap();
        let pos = pos_store.get_component(e1.index).unwrap();
        let vel = vel_store.get_component(e1.index).unwrap();
        println!("frame {frame:>2}: pos ({:>6.1},{:>6.1})  vel ({:>6.1},{:>6.1})",
                 pos.0, pos.1, vel.0, vel.1);
    }
}

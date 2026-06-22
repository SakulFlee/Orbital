/// Stress-test parallelism with many non-conflicting systems.
///
/// Run: cargo run --example parallel --release
use orbital_ecs::{Schedule, World};

const ENTITIES: usize = 100_000;

#[derive(Debug, Clone)]
struct A(i32);
#[derive(Debug, Clone)]
struct B(i32);
#[derive(Debug, Clone)]
struct C(i32);
#[derive(Debug, Clone)]
struct D(i32);
#[derive(Debug, Clone)]
struct E(i32);
#[derive(Debug, Clone)]
struct F(i32);
#[derive(Debug, Clone)]
struct G(i32);
#[derive(Debug, Clone)]
struct H(i32);

fn inc_a(a: &mut A) { a.0 += 1; }
fn inc_b(b: &mut B) { b.0 += 1; }
fn inc_c(c: &mut C) { c.0 += 1; }
fn inc_d(d: &mut D) { d.0 += 1; }
fn inc_e(e: &mut E) { e.0 += 1; }
fn inc_f(f: &mut F) { f.0 += 1; }
fn inc_g(g: &mut G) { g.0 += 1; }
fn inc_h(h: &mut H) { h.0 += 1; }

fn main() {
    let mut world = World::new();
    for _ in 0..ENTITIES {
        let e = world.spawn_entity();
        world.attach_component(&e, A(0)).unwrap();
        world.attach_component(&e, B(0)).unwrap();
        world.attach_component(&e, C(0)).unwrap();
        world.attach_component(&e, D(0)).unwrap();
        world.attach_component(&e, E(0)).unwrap();
        world.attach_component(&e, F(0)).unwrap();
        world.attach_component(&e, G(0)).unwrap();
        world.attach_component(&e, H(0)).unwrap();
    }

    let mut schedule = Schedule::new();
    schedule.add_system(inc_a);
    schedule.add_system(inc_b);
    schedule.add_system(inc_c);
    schedule.add_system(inc_d);
    schedule.add_system(inc_e);
    schedule.add_system(inc_f);
    schedule.add_system(inc_g);
    schedule.add_system(inc_h);

    schedule.run(&world);

    // Verify via store handles.
    let store = world.get_component_store::<A>().unwrap();
    for id in [0, 1, ENTITIES - 1] {
        let a = store.get_component(id).unwrap();
        println!("entity {id}: A = {}", a.0);
    }
}

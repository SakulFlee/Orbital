use crate::shapes::plane;
use cgmath::Vector2;
use orbital_resources::MeshDescriptor;

pub fn grid(width: f32, depth: f32, cols: u32, rows: u32) -> MeshDescriptor {
    plane(Vector2::new(width, depth), cols.max(rows))
}

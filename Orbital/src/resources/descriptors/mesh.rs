use cgmath::Point3;
use vertex::Vertex;

use super::BoundingBoxDescriptor;

#[derive(Debug, Clone, Eq, Hash)]
pub struct MeshDescriptor {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub bounding_box: BoundingBoxDescriptor,
}

impl MeshDescriptor {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        let mut min = Point3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut max = Point3::new(f32::MIN, f32::MIN, f32::MIN);
        for vertex in &vertices {
            let position = vertex.position;
            min = Point3::new(
                min.x.min(position.x),
                min.y.min(position.y),
                min.z.min(position.z),
            );
            max = Point3::new(
                max.x.max(position.x),
                max.y.max(position.y),
                max.z.max(position.z),
            );
        }
        let bounding_box = BoundingBoxDescriptor { min, max };

        Self {
            vertices,
            indices,
            bounding_box,
        }
    }
}

impl PartialEq for MeshDescriptor {
    fn eq(&self, other: &Self) -> bool {
        // First compare lengths
        if self.vertices.len() != other.vertices.len() || self.indices.len() != other.indices.len()
        {
            return false;
        }

        // Then compare the actual data
        self.vertices == other.vertices && self.indices == other.indices
    }
}

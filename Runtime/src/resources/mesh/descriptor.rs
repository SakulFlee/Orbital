use cgmath::Point3;

use crate::resources::{BoundingBoxDescriptor, Vertex};

#[derive(Debug, Clone, Eq, Hash)]
pub struct MeshDescriptor {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl MeshDescriptor {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        Self { vertices, indices }
    }

    pub fn find_bounding_box(&self) -> BoundingBoxDescriptor {
        let mut min = Point3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut max = Point3::new(f32::MIN, f32::MIN, f32::MIN);
        for vertex in &self.vertices {
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
        BoundingBoxDescriptor { min, max }
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

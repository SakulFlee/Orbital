use std::hash::Hash;

use crate::Vertex;

#[derive(Debug, Clone, Eq)]
pub struct MeshDescriptor {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl MeshDescriptor {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        Self { vertices, indices }
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

impl Hash for MeshDescriptor {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.vertices.hash(state);
        self.indices.hash(state);
    }
}

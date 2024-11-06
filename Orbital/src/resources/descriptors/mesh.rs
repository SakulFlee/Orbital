use crate::resources::realizations::Vertex;

#[derive(Debug, Clone)]
pub struct MeshDescriptor {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
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

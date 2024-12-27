use crate::{bounding_box::BoundingBox, resources::realizations::Vertex};

#[derive(Debug, Clone, Eq, Hash)]
pub struct MeshDescriptor {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    /// Bounding box of the mesh.
    /// Set this to `Some` to set a bounding box.
    /// Set this to `None` to specifically not set a bounding box.
    ///
    /// Bounding boxes are used in Orbital in many places.
    /// However, it's main usage is in the `Renderer` for culling.
    /// Any `Mesh` without a bounding box will **always** be rendered and never be culled.
    /// This is useful for debugging and giant objects that, probably, are always on screen.
    /// Any other mesh should have a bounding box!
    pub bounding_box: Option<BoundingBox<f32>>,
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

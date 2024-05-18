use crate::resources::realizations::Vertex;

#[derive(Debug, Clone)]
pub struct MeshDescriptor {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

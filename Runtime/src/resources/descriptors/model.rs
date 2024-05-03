use super::{MaterialDescriptor, MeshDescriptor};

#[derive(Debug, Clone)]
pub struct ModelDescriptor {
    pub mesh_descriptor: MeshDescriptor,
    pub material_descriptor: MaterialDescriptor,
}

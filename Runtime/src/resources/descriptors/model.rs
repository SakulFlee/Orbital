use super::{MaterialDescriptor, MeshDescriptor};

pub struct ModelDescriptor<'a> {
    pub mesh_descriptor: MeshDescriptor,
    pub material_descriptor: MaterialDescriptor<'a>,
}

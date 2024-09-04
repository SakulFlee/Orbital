use easy_gltf::Model;

use crate::resources::descriptors::{InstanceDescriptor, Instancing, ModelDescriptor};

impl From<&Model> for ModelDescriptor {
    fn from(gltf_model: &Model) -> Self {
        let material_descriptor = gltf_model.material().as_ref().into();
        let mesh_descriptor = gltf_model.into();

        ModelDescriptor::FromDescriptors(
            mesh_descriptor,
            material_descriptor,
            Instancing::Single(InstanceDescriptor::default()),
        )
    }
}

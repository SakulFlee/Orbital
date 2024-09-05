use easy_gltf::Model;

use crate::resources::descriptors::{InstanceDescriptor, Instancing, ModelDescriptor};

impl From<&Model> for ModelDescriptor {
    fn from(gltf_model: &Model) -> Self {
        let material = gltf_model.material().as_ref().into();
        let mesh = gltf_model.into();

        ModelDescriptor::FromDescriptors {
            mesh,
            material,
            instancing: Instancing::Single(InstanceDescriptor::default()),
        }
    }
}

use easy_gltf::Model;

use crate::resources::descriptors::{InstanceDescriptor, Instancing, ModelDescriptor};

impl From<&Model> for ModelDescriptor {
    fn from(gltf_model: &Model) -> Self {
        let label = gltf_model.mesh_name().map(|x| x.to_string()).unwrap_or(String::from("unlabelled glTF Model"));

        let material = gltf_model.material().as_ref().into();
        let mesh = gltf_model.into();

        ModelDescriptor::FromDescriptors {
            label,
            mesh,
            material,
            instancing: Instancing::Single(InstanceDescriptor::default()),
        }
    }
}

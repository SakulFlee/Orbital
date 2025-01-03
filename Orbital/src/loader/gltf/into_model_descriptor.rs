use std::sync::Arc;

use easy_gltf::Model;

use crate::{resources::descriptors::ModelDescriptor, transform::Transform};

impl From<&Model> for ModelDescriptor {
    fn from(gltf_model: &Model) -> Self {
        let label = gltf_model
            .mesh_name()
            .map(|x| x.to_string())
            .unwrap_or(String::from("unlabelled glTF Model"));

        let material = Arc::new(gltf_model.material().as_ref().into());
        let mesh = Arc::new(gltf_model.into());

        ModelDescriptor {
            label,
            mesh,
            material,
            transforms: vec![Transform::default()], // TODO
        }
    }
}

use std::sync::Arc;

use easy_gltf::Model;
use wgpu::Color;

use crate::{
    resources::descriptors::{MaterialDescriptor, ModelDescriptor},
    transform::Transform,
};

impl From<&Model> for ModelDescriptor {
    fn from(gltf_model: &Model) -> Self {
        let label = gltf_model
            .mesh_name()
            .map(|x| x.to_string())
            .unwrap_or(String::from("unlabelled glTF Model"));

        let material = Arc::new(gltf_model.material().as_ref().into());
        let mesh = Arc::new(gltf_model.into());

        let wireframe_material = Arc::new(MaterialDescriptor::Wireframe(Color::WHITE));

        ModelDescriptor {
            label,
            mesh,
            materials: vec![material, wireframe_material],
            transforms: vec![Transform::default()], // TODO: This only works because vertices seem to already be offset by the correct amount for their local space. We should find out if this is from easy_gltf, or, encoded into glTF directly by Blender. Either way, it might be best to "re-local-ize" the vertices to reduce number overhead and properly use transforms.
            #[cfg(debug_assertions)]
            render_bounding_box: false,
        }
    }
}

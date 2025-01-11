use std::sync::Arc;

use easy_gltf::Model;

use crate::{
    resources::descriptors::{ModelDescriptor, RenderMode},
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

        ModelDescriptor {
            label,
            mesh,
            material,
            transforms: vec![Transform::default()], // TODO: This only works because vertices seem to already be offset by the correct amount for their local space. We should find out if this is from easy_gltf, or, encoded into glTF directly by Blender. Either way, it might be best to "re-local-ize" the vertices to reduce number overhead and properly use transforms.
            render_modes: RenderMode::Solid,
            #[cfg(debug_assertions)]
            render_bounding_box: false,
        }
    }
}

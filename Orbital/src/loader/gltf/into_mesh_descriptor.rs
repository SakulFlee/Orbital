use easy_gltf::Model;

use crate::resources::{descriptors::MeshDescriptor, realizations::Vertex};

impl From<&Model> for MeshDescriptor {
    fn from(gltf_model: &Model) -> Self {
        let vertices = gltf_model
            .vertices()
            .iter()
            .map(|vertex| Into::<Vertex>::into(*vertex))
            .collect::<Vec<Vertex>>();

        let indices = gltf_model
            .indices()
            .expect("Trying to load glTF model without Indices!")
            .clone();

        MeshDescriptor { vertices, indices }
    }
}

use cgmath::Point3;
use easy_gltf::Model;

use crate::{
    bounding_box::BoundingBox,
    resources::{descriptors::MeshDescriptor, realizations::Vertex},
};

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

        // Bounding Box
        let mut min = Point3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut max = Point3::new(f32::MIN, f32::MIN, f32::MIN);

        for vertex in &vertices {
            let position = vertex.position;
            min = Point3::new(
                min.x.min(position.x),
                min.y.min(position.y),
                min.z.min(position.z),
            );
            max = Point3::new(
                max.x.max(position.x),
                max.y.max(position.y),
                max.z.max(position.z),
            );
        }

        let bounding_box = Some(BoundingBox { a: min, b: max });

        MeshDescriptor {
            vertices,
            indices,
            bounding_box,
        }
    }
}

use easy_gltf::model::Vertex;

use crate::engine::VertexPoint;

impl From<&Vertex> for VertexPoint {
    /// Note: tangent and bitangent are NOT set!
    fn from(value: &Vertex) -> Self {
        Self {
            position_coordinates: value.position.into(),
            texture_coordinates: value.tex_coords.into(),
            normal_coordinates: value.normal.into(),
            tangent: [0.0; 3],
            bitangent: [0.0; 3],
        }
    }
}

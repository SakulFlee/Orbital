use easy_gltf::model::Vertex;

use crate::engine::VertexPoint;

impl From<&Vertex> for VertexPoint {
    fn from(value: &Vertex) -> Self {
        // TODO: Tangent?
        Self {
            position_coordinates: value.position.into(),
            texture_coordinates: value.tex_coords.into(),
            normal_coordinates: value.normal.into(),
        }
    }
}

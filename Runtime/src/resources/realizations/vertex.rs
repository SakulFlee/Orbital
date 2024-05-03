use crate::{nalgebra::Vector3, resources::uniforms::VertexUniform};

#[derive(Debug, Clone)]
pub struct Vertex {
    pub position_coordinates: Vector3<f32>,
}

impl From<&Vertex> for VertexUniform {
    fn from(value: &Vertex) -> Self {
        VertexUniform {
            positional_coordinates: value.position_coordinates.into(),
        }
    }
}

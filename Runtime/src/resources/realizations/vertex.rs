use crate::{nalgebra::Vector3, resources::uniforms::VertexUniform};

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

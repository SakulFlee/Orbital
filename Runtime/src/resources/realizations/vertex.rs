use std::mem::size_of;

use cgmath::{Vector2, Vector3};
use wgpu::{VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

#[derive(Debug, Clone)]
pub struct Vertex {
    pub position_coordinates: Vector3<f32>,
    pub texture_coordinates: Vector2<f32>,
}

impl Vertex {
    pub fn vertex_buffer_layout_descriptor() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: size_of::<[f32; 5]>() as u64,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x3,
                },
                VertexAttribute {
                    offset: size_of::<[f32; 3]>() as u64,
                    shader_location: 1,
                    format: VertexFormat::Float32x2,
                },
            ],
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        vec![
            self.position_coordinates.x.to_le_bytes(),
            self.position_coordinates.y.to_le_bytes(),
            self.position_coordinates.z.to_le_bytes(),
            self.texture_coordinates.x.to_le_bytes(),
            self.texture_coordinates.y.to_le_bytes(),
        ]
        .concat()
    }
}

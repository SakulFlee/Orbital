use std::mem::size_of;

use bytemuck::{Pod, Zeroable};
use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

use crate::resources::Vertex;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct VertexUniform {
    pub positional_coordinates: [f32; 3],
    pub texture_coordinates: [f32; 2],
}

impl VertexUniform {
    pub fn vertex_buffer_layout_descriptor() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: size_of::<VertexUniform>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x3,
                },
                VertexAttribute {
                    offset: size_of::<[f32; 3]>() as BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x2,
                },
            ],
        }
    }
}

impl From<&VertexUniform> for Vertex {
    fn from(value: &VertexUniform) -> Self {
        Self {
            position_coordinates: value.positional_coordinates.into(),
            texture_coordinates: value.texture_coordinates.into(),
        }
    }
}

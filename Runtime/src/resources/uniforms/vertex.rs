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
    
}

impl From<&VertexUniform> for Vertex {
    fn from(value: &VertexUniform) -> Self {
        Self {
            position_coordinates: value.positional_coordinates.into(),
            texture_coordinates: value.texture_coordinates.into(),
        }
    }
}

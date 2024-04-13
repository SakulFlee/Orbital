use std::mem::size_of;

use bytemuck::{Pod, Zeroable};
use nalgebra::Vector3;
use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

// --- Normal Type ---

#[derive(Debug, Copy, Clone)]
pub struct Vertex {
    pub position_coordinates: Vector3<f32>,
}

impl Vertex {
    pub fn descriptor() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: size_of::<VertexRaw>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x3,
                },
            ],
        }
    }
}

// --- Raw Type ---

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct VertexRaw {
    pub position_coordinates: [f32; 3],
}

impl From<Vertex> for VertexRaw {
    fn from(value: Vertex) -> Self {
        Self {
            position_coordinates: value.position_coordinates.as_slice().try_into().unwrap(),
        }
    }
}

impl From<&Vertex> for VertexRaw {
    fn from(value: &Vertex) -> Self {
        Self {
            position_coordinates: value.position_coordinates.as_slice().try_into().unwrap(),
        }
    }
}

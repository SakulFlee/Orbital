use std::mem::size_of;

use cgmath::{Vector2, Vector3};
use wgpu::{VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

#[derive(Debug, Clone)]
pub struct Vertex {
    pub position: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub tangent: Vector3<f32>,
    pub bitangent: Vector3<f32>,
    pub uv: Vector2<f32>,
}

impl Vertex {
    pub fn vertex_buffer_layout_descriptor() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: size_of::<[f32; 3 * 4 + 2]>() as u64,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                // Position
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x3,
                },
                // Normal
                VertexAttribute {
                    offset: size_of::<[f32; 3]>() as u64,
                    shader_location: 1,
                    format: VertexFormat::Float32x3,
                },
                // Tangent
                VertexAttribute {
                    offset: size_of::<[f32; 3 * 2]>() as u64,
                    shader_location: 2,
                    format: VertexFormat::Float32x3,
                },
                // Bitangent
                VertexAttribute {
                    offset: size_of::<[f32; 3 * 3]>() as u64,
                    shader_location: 3,
                    format: VertexFormat::Float32x3,
                },
                // UV
                VertexAttribute {
                    offset: size_of::<[f32; 3 * 4]>() as u64,
                    shader_location: 4,
                    format: VertexFormat::Float32x2,
                },
            ],
        }
    }

    pub fn new(
        position: Vector3<f32>,
        normal: Vector3<f32>,
        tangent: Vector3<f32>,
        uv: Vector2<f32>,
    ) -> Self {
        Self {
            position,
            normal,
            tangent,
            bitangent: Self::calculate_binormal(tangent, normal),
            uv,
        }
    }

    pub fn calculate_binormal(tangent: Vector3<f32>, normal: Vector3<f32>) -> Vector3<f32> {
        tangent.cross(normal)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        [
            self.position.x.to_le_bytes(),
            self.position.y.to_le_bytes(),
            self.position.z.to_le_bytes(),
            self.normal.x.to_le_bytes(),
            self.normal.y.to_le_bytes(),
            self.normal.z.to_le_bytes(),
            self.tangent.x.to_le_bytes(),
            self.tangent.y.to_le_bytes(),
            self.tangent.z.to_le_bytes(),
            self.bitangent.x.to_le_bytes(),
            self.bitangent.y.to_le_bytes(),
            self.bitangent.z.to_le_bytes(),
            self.uv.x.to_le_bytes(),
            self.uv.y.to_le_bytes(),
        ]
        .concat()
    }
}

impl From<easy_gltf::model::Vertex> for Vertex {
    fn from(value: easy_gltf::model::Vertex) -> Self {
        Self::new(
            value.position,
            value.normal,
            // Note: Skipping `w` which defines the strength of the vector.
            Vector3::new(value.tangent.x, value.tangent.y, value.tangent.z),
            value.tex_coords,
        )
    }
}

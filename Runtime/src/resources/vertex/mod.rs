use std::{hash::Hash, mem::size_of};

use cgmath::{num_traits::Float, Vector2, Vector3};
use wgpu::{VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

#[derive(Debug, Clone, PartialEq)]
pub struct Vertex {
    pub position: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub tangent: Vector3<f32>,
    pub bitangent: Vector3<f32>,
    pub uv: Vector2<f32>,
}

impl Vertex {
    pub fn complex_vertex_buffer_layout_descriptor() -> VertexBufferLayout<'static> {
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

    pub fn simple_vertex_buffer_layout_descriptor() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: size_of::<[f32; 3]>() as u64,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                // Position
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x3,
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

/// Note: This ignores that f32 can't be Eq'd by default due to NaN.
/// Vertices should **never** be using NaN.
impl Eq for Vertex {}

impl Hash for Vertex {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.position.x.integer_decode().hash(state);
        self.position.y.integer_decode().hash(state);
        self.position.z.integer_decode().hash(state);

        self.normal.x.integer_decode().hash(state);
        self.normal.y.integer_decode().hash(state);
        self.normal.z.integer_decode().hash(state);

        self.tangent.x.integer_decode().hash(state);
        self.tangent.y.integer_decode().hash(state);
        self.tangent.z.integer_decode().hash(state);

        self.bitangent.x.integer_decode().hash(state);
        self.bitangent.y.integer_decode().hash(state);
        self.bitangent.z.integer_decode().hash(state);

        self.uv.x.integer_decode().hash(state);
        self.uv.y.integer_decode().hash(state);
    }
}

/// A `Vertex` is already realized after describing it.
/// Thus, it's both a _descriptor_ and a _realization_.
///
/// # Why?
/// _Realizing_ a _descriptor_ involves **creating** (or: generating) the given resource and commonly putting the result into a e.g. Buffer to be used by the GPU at some point.
///
/// It doesn't make sense to have a buffer per-vertex, rather bundle them together into a **Model**.
/// Furthermore, there isn't really any conversion or creation happening.
/// The `Vertex` is already completely realized as it is described.
///
/// This certainly is a unique case!
pub type VertexDescriptor = Vertex;

#[cfg(feature = "gltf")]
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

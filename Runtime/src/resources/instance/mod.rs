use std::mem;

use cgmath::Matrix4;
use wgpu::{VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

use super::Transform;

mod descriptor;
pub use descriptor::*;

mod instancing;
pub use instancing::*;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Instance {
    descriptor: InstanceDescriptor,
}

impl Instance {
    pub fn from_descriptor(instance_descriptor: &InstanceDescriptor) -> Self {
        Self {
            descriptor: *instance_descriptor,
        }
    }

    pub fn make_model_space_matrix(&self) -> Matrix4<f32> {
        let matrix_position = Matrix4::from_translation(self.descriptor.position);

        let matrix_rotation = Matrix4::from(self.descriptor.rotation);

        let matrix_scale = Matrix4::from_nonuniform_scale(
            self.descriptor.scale.x,
            self.descriptor.scale.y,
            self.descriptor.scale.z,
        );

        matrix_position * matrix_rotation * matrix_scale
    }

    pub fn vertex_buffer_layout_descriptor() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: mem::size_of::<[f32; 4 * 4]>() as u64,
            step_mode: VertexStepMode::Instance,
            // A Matrix4x4<f32> is the same as 4x Vector4<f32>.
            // Thus, 4x slots.
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as u64,
                    shader_location: 6,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: mem::size_of::<[f32; 4 * 2]>() as u64,
                    shader_location: 7,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: mem::size_of::<[f32; 4 * 3]>() as u64,
                    shader_location: 8,
                    format: VertexFormat::Float32x4,
                },
            ],
        }
    }

    pub fn to_buffer_data(&self) -> Vec<[u8; 4]> {
        let matrix = self.make_model_space_matrix();

        vec![
            matrix.x.x.to_le_bytes(),
            matrix.x.y.to_le_bytes(),
            matrix.x.z.to_le_bytes(),
            matrix.x.w.to_le_bytes(),
            matrix.y.x.to_le_bytes(),
            matrix.y.y.to_le_bytes(),
            matrix.y.z.to_le_bytes(),
            matrix.y.w.to_le_bytes(),
            matrix.z.x.to_le_bytes(),
            matrix.z.y.to_le_bytes(),
            matrix.z.z.to_le_bytes(),
            matrix.z.w.to_le_bytes(),
            matrix.w.x.to_le_bytes(),
            matrix.w.y.to_le_bytes(),
            matrix.w.z.to_le_bytes(),
            matrix.w.w.to_le_bytes(),
        ]
    }

    pub fn to_buffer_data_flattened(&self) -> Vec<u8> {
        self.to_buffer_data().into_flattened()
    }
}

impl From<&Transform> for Instance {
    fn from(value: &Transform) -> Self {
        Self {
            descriptor: value.into(),
        }
    }
}

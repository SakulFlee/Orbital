use std::mem;

use crate::resources::descriptors::InstanceDescriptor;
use cgmath::Matrix4;
use wgpu::{VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

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
                    shader_location: 2,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as u64,
                    shader_location: 3,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as u64,
                    shader_location: 4,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as u64,
                    shader_location: 5,
                    format: VertexFormat::Float32x4,
                },
            ],
        }
    }
}

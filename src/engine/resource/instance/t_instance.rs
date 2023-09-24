use cgmath::{Matrix4, Quaternion, Vector3};
use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

use super::InstanceUniform;

pub trait TInstance {
    fn new(position: Vector3<f32>, rotation: Quaternion<f32>) -> Self;

    fn get_position(&self) -> Vector3<f32>;

    fn set_position(&mut self, postion: Vector3<f32>);

    fn get_rotation(&self) -> Quaternion<f32>;

    fn set_rotation(&mut self, rotation: Quaternion<f32>);

    fn to_instance_uniform(&self) -> InstanceUniform {
        InstanceUniform {
            model_space_matrix: (Matrix4::from_translation(self.get_position())
                * Matrix4::from(self.get_rotation()))
            .into(),
        }
    }

    fn descriptor() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceUniform>() as BufferAddress,
            step_mode: VertexStepMode::Instance,
            // A Matrix4x4 is 4x Vec[4] of floats, so we need 4x slots
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as BufferAddress,
                    shader_location: 6,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 8]>() as BufferAddress,
                    shader_location: 7,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 12]>() as BufferAddress,
                    shader_location: 8,
                    format: VertexFormat::Float32x4,
                },
            ],
        }
    }
}

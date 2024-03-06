use cgmath::{Matrix3, Matrix4, Quaternion, Vector3};
use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

use super::InstanceUniform;

pub trait TInstance {
    fn new(position: Vector3<f32>, rotation: Quaternion<f32>) -> Self;

    fn position(&self) -> Vector3<f32>;

    fn set_position(&mut self, postion: Vector3<f32>);

    fn rotation(&self) -> Quaternion<f32>;

    fn set_rotation(&mut self, rotation: Quaternion<f32>);

    fn to_instance_uniform(&self) -> InstanceUniform {
        InstanceUniform {
            model_space_matrix: (Matrix4::from_translation(self.position())
                * Matrix4::from(self.rotation()))
            .into(),
            normal_space_matrix: Matrix3::from(self.rotation()).into(),
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
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 16]>() as BufferAddress,
                    shader_location: 9,
                    format: VertexFormat::Float32x3,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 19]>() as BufferAddress,
                    shader_location: 10,
                    format: VertexFormat::Float32x3,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 22]>() as BufferAddress,
                    shader_location: 11,
                    format: VertexFormat::Float32x3,
                },
            ],
        }
    }
}

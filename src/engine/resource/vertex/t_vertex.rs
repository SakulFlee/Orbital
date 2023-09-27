use std::mem::size_of;

use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

pub trait TVertex {
    fn position_coordinates(&self) -> [f32; 3];
    fn texture_coordinates(&self) -> [f32; 2];
    fn normal_coordinates(&self) -> [f32; 3];

    fn descriptor<T>() -> VertexBufferLayout<'static>
    where
        Self: Sized,
        T: TVertex,
    {
        VertexBufferLayout {
            array_stride: size_of::<T>() as BufferAddress,
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
                VertexAttribute {
                    offset: size_of::<[f32; 5]>() as BufferAddress,
                    shader_location: 2,
                    format: VertexFormat::Float32x3,
                },
            ],
        }
    }
}

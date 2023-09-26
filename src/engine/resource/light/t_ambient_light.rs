use cgmath::Vector3;
use wgpu::{
    BindGroup, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
    Buffer, BufferBindingType, Device, Queue, ShaderStages,
};

use super::AmbientLightUniform;

pub trait TAmbientLight {
    const BIND_GROUP_LAYOUT_DESCRIPTOR: BindGroupLayoutDescriptor<'static> =
        BindGroupLayoutDescriptor {
            label: Some("Ambient Light Bind Group Layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        };

    fn update_buffer(&self, queue: &Queue) {
        queue.write_buffer(
            &self.get_buffer(),
            0,
            bytemuck::cast_slice(&[self.to_uniform()]),
        )
    }

    fn to_uniform(&self) -> AmbientLightUniform;

    fn get_color(&self) -> Vector3<f32>;

    fn set_color(&mut self, color: Vector3<f32>);

    fn get_strength(&self) -> f32;

    fn set_strength(&mut self, strength: f32);

    fn get_bind_group_layout(device: &Device) -> BindGroupLayout;

    fn get_buffer(&self) -> &Buffer;

    fn get_bind_group(&self) -> &BindGroup;
}

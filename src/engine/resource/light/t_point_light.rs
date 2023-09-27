use cgmath::Vector3;
use wgpu::{
    BindGroup, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
    Buffer, BufferBindingType, ShaderStages,
};

use crate::engine::LogicalDevice;

use super::PointLightUniform;

pub trait TPointLight {
    const BIND_GROUP_LAYOUT_DESCRIPTOR: BindGroupLayoutDescriptor<'static> =
        BindGroupLayoutDescriptor {
            label: Some("Point Light Bind Group Layout"),
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

    fn update_buffer(&mut self, logical_device: &LogicalDevice) {
        logical_device.queue().write_buffer(
            self.buffer(),
            0,
            bytemuck::cast_slice(&[self.to_uniform()]),
        )
    }

    fn to_uniform(&self) -> PointLightUniform;

    fn color(&self) -> Vector3<f32>;

    fn set_color(&mut self, color: Vector3<f32>);

    fn position(&self) -> Vector3<f32>;

    fn set_position(&mut self, position: Vector3<f32>);

    fn strength(&self) -> f32;

    fn set_strength(&mut self, strength: f32);

    fn enabled(&self) -> bool;

    fn set_enabled(&mut self, enabled: bool);

    fn bind_group_layout(logical_device: &LogicalDevice) -> BindGroupLayout {
        logical_device
            .device()
            .create_bind_group_layout(&Self::BIND_GROUP_LAYOUT_DESCRIPTOR)
    }

    fn buffer(&self) -> &Buffer;

    fn bind_group(&self) -> &BindGroup;
}

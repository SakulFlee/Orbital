use cgmath::Vector3;
use wgpu::{BindGroup, BindGroupLayout, Buffer, Device, Queue};

use super::PointLightUniform;

pub trait TPointLight {
    fn update_buffer(&mut self, queue: &Queue) {
        queue.write_buffer(
            &self.get_buffer(),
            0,
            bytemuck::cast_slice(&[self.to_uniform()]),
        )
    }

    fn to_uniform(&self) -> PointLightUniform;

    fn get_color(&self) -> Vector3<f32>;

    fn set_color(&mut self, color: Vector3<f32>);

    fn get_position(&self) -> Vector3<f32>;

    fn set_position(&mut self, position: Vector3<f32>);

    fn get_strength(&self) -> f32;

    fn set_strength(&mut self, strength: f32);

    fn get_enabled(&self) -> bool;

    fn set_enabled(&mut self, enabled: bool);

    fn get_bind_group_layout(device: &Device) -> BindGroupLayout;

    fn get_buffer(&self) -> &Buffer;

    fn get_bind_group(&self) -> &BindGroup;
}

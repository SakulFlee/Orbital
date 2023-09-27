use cgmath::Vector3;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, Buffer, BufferUsages,
};

use crate::engine::LogicalDevice;

use super::{TAmbientLight, UAmbientLight};

pub struct StandardAmbientLight {
    color: Vector3<f32>,
    strength: f32,
    buffer: Buffer,
    bind_group: BindGroup,
}

impl StandardAmbientLight {
    pub fn new(logical_device: &LogicalDevice, color: Vector3<f32>, strength: f32) -> Self {
        let empty_uniform: UAmbientLight = UAmbientLight::empty();
        let buffer = logical_device
            .device()
            .create_buffer_init(&BufferInitDescriptor {
                label: Some("Ambient Light Buffer"),
                contents: bytemuck::cast_slice(&[empty_uniform]),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            });

        let bind_group_layout = Self::bind_group_layout(logical_device);
        let bind_group = logical_device
            .device()
            .create_bind_group(&BindGroupDescriptor {
                label: Some("Ambient Light Bind Group"),
                layout: &bind_group_layout,
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
            });

        let light = Self {
            color,
            strength,
            buffer,
            bind_group,
        };

        light.update_buffer(logical_device);

        light
    }
}

impl TAmbientLight for StandardAmbientLight {
    fn to_uniform(&self) -> UAmbientLight {
        UAmbientLight::new(self.color.into(), self.strength)
    }

    fn color(&self) -> Vector3<f32> {
        self.color
    }

    fn set_color(&mut self, color: Vector3<f32>) {
        self.color = color;
    }

    fn strength(&self) -> f32 {
        self.strength
    }

    fn set_strength(&mut self, strength: f32) {
        self.strength = strength;
    }

    fn bind_group_layout(logical_device: &LogicalDevice) -> BindGroupLayout {
        logical_device
            .device()
            .create_bind_group_layout(&Self::BIND_GROUP_LAYOUT_DESCRIPTOR)
    }

    fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}

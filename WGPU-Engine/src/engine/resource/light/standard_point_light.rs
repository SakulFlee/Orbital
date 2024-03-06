use cgmath::Vector3;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupDescriptor, BindGroupEntry, Buffer, BufferUsages,
};

use crate::engine::LogicalDevice;

use super::{TPointLight, UPointLight};

pub struct StandardPointLight {
    color: Vector3<f32>,
    position: Vector3<f32>,
    strength: f32,
    enabled: bool,
    buffer: Buffer,
    bind_group: BindGroup,
}

impl StandardPointLight {
    pub fn new(
        logical_device: &LogicalDevice,
        color: Vector3<f32>,
        position: Vector3<f32>,
        strength: f32,
        enabled: bool,
    ) -> Self {
        let empty_uniform = UPointLight::empty();
        let buffer = logical_device
            .device()
            .create_buffer_init(&BufferInitDescriptor {
                label: Some("Point Light Buffer"),
                contents: bytemuck::cast_slice(&[empty_uniform]),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            });

        let bind_group_layout = Self::bind_group_layout(logical_device);
        let bind_group = logical_device
            .device()
            .create_bind_group(&BindGroupDescriptor {
                label: Some("Point Light Bind Group"),
                layout: &bind_group_layout,
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
            });

        let mut light = Self {
            color,
            position,
            strength,
            enabled,
            buffer,
            bind_group,
        };

        light.update_buffer(logical_device);

        light
    }
}

impl TPointLight for StandardPointLight {
    fn to_uniform(&self) -> UPointLight {
        UPointLight::new(
            self.color.into(),
            self.position.into(),
            self.strength,
            self.enabled,
        )
    }

    fn color(&self) -> Vector3<f32> {
        self.color
    }

    fn set_color(&mut self, color: Vector3<f32>) {
        self.color = color;
    }

    fn position(&self) -> Vector3<f32> {
        self.position
    }

    fn set_position(&mut self, position: Vector3<f32>) {
        self.position = position;
    }

    fn strength(&self) -> f32 {
        self.strength
    }

    fn set_strength(&mut self, strength: f32) {
        self.strength = strength;
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}

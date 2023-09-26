use cgmath::Vector3;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupDescriptor, BindGroupEntry, Buffer, BufferUsages, Device, Queue,
};

use super::{PointLightUniform, TPointLight};

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
        device: &Device,
        queue: &Queue,
        color: Vector3<f32>,
        position: Vector3<f32>,
        strength: f32,
        enabled: bool,
    ) -> Self {
        let empty_uniform = PointLightUniform::empty();
        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Point Light Buffer"),
            contents: bytemuck::cast_slice(&[empty_uniform]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let bind_group_layout = Self::get_bind_group_layout(device);
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
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

        light.update_buffer(queue);

        light
    }
}

impl TPointLight for StandardPointLight {
    fn to_uniform(&self) -> PointLightUniform {
        PointLightUniform::new(
            self.color.into(),
            self.position.into(),
            self.strength,
            self.enabled,
        )
    }

    fn get_color(&self) -> Vector3<f32> {
        self.color
    }

    fn set_color(&mut self, color: Vector3<f32>) {
        self.color = color;
    }

    fn get_position(&self) -> Vector3<f32> {
        self.position
    }

    fn set_position(&mut self, position: Vector3<f32>) {
        self.position = position;
    }

    fn get_strength(&self) -> f32 {
        self.strength
    }

    fn set_strength(&mut self, strength: f32) {
        self.strength = strength;
    }

    fn get_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn get_buffer(&self) -> &Buffer {
        &self.buffer
    }

    fn get_bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}

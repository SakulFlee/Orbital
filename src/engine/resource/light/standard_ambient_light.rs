use cgmath::Vector3;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferUsages, Device, Queue,
    ShaderStages,
};

use super::{AmbientLightUniform, TAmbientLight};

pub struct StandardAmbientLight {
    color: Vector3<f32>,
    strength: f32,
    buffer: Buffer,
    bind_group: BindGroup,
}

impl StandardAmbientLight {
    pub const BIND_GROUP_LAYOUT_DESCRIPTOR: BindGroupLayoutDescriptor<'static> =
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

    pub fn new(device: &Device, queue: &Queue, color: Vector3<f32>, strength: f32) -> Self {
        let empty_uniform = AmbientLightUniform::empty();
        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Ambient Light Buffer"),
            contents: bytemuck::cast_slice(&[empty_uniform]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let bind_group_layout = Self::get_bind_group_layout(device);
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
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

        light.update_buffer(queue);

        light
    }
}

impl TAmbientLight for StandardAmbientLight {
    fn to_uniform(&self) -> AmbientLightUniform {
        AmbientLightUniform::new(self.color.into(), self.strength)
    }

    fn get_color(&self) -> Vector3<f32> {
        self.color
    }

    fn set_color(&mut self, color: Vector3<f32>) {
        self.color = color;
    }

    fn get_strength(&self) -> f32 {
        self.strength
    }

    fn set_strength(&mut self, strength: f32) {
        self.strength = strength;
    }

    fn get_bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&Self::BIND_GROUP_LAYOUT_DESCRIPTOR)
    }

    fn get_buffer(&self) -> &Buffer {
        &self.buffer
    }

    fn get_bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}

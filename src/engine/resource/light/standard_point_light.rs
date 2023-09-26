use cgmath::Vector3;
use wgpu::{BindGroup, BindGroupLayout, Buffer, Device};

use super::{PointLightUniform, TPointLight};

pub struct StandardPointLight {
    color: Vector3<f32>,
    position: Vector3<f32>,
    strength: f32,
    enabled: bool,
}

impl StandardPointLight {
    pub fn new(color: Vector3<f32>, position: Vector3<f32>, strength: f32, enabled: bool) -> Self {
        Self {
            color,
            position,
            strength,
            enabled,
        }
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

    fn get_bind_group(&self) -> &BindGroup {
        todo!()
    }

    fn get_bind_group_layout(device: &Device) -> BindGroupLayout {
        todo!()
    }

    fn get_buffer(&self) -> &Buffer {
        todo!()
    }
}

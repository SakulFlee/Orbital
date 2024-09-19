use wgpu::{BindGroup, Buffer, Device, Queue};

use crate::resources::{descriptors::LightDescriptor, realizations::PointLight};

mod point;
pub use point::*;

#[derive(Debug)]
pub struct LightStore {
    point_light_store: PointLightStore,
}

impl LightStore {
    pub fn new() -> Self {
        Self {
            point_light_store: PointLightStore::new(),
        }
    }

    pub fn update_if_needed(&mut self, device: &Device, queue: &Queue) {
        self.point_light_store.update_if_needed(device, queue);
    }

    pub fn add_light_descriptor(&mut self, light_descriptor: LightDescriptor) {
        match light_descriptor {
            LightDescriptor::PointLight(point_light) => {
                self.point_light_store.add_point_light(point_light)
            }
        }
    }

    pub fn add_point_light(&mut self, point_light: PointLight) {
        self.point_light_store.add_point_light(point_light);
    }

    pub fn remove_any_light_with_label(&mut self, label: &str) {
        self.point_light_store.remove_point_light(label);
    }

    pub fn remove_point_light(&mut self, label: &str) {
        self.point_light_store.remove_point_light(label);
    }

    pub fn point_light_buffer(&self) -> &Buffer {
        self.point_light_store.buffer()
    }

    pub fn point_light_bind_group(&self) -> &BindGroup {
        self.point_light_store.bind_group()
    }

    pub fn clear(&mut self) {
        self.point_light_store.clear();
    }
}

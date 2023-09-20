use wgpu::{BindGroup, BindGroupLayout, Device};

use crate::engine::DiffuseTexture;

pub trait TMaterial {
    fn get_name(&self) -> &str;
    fn get_diffuse_texture(&self) -> &DiffuseTexture;
    fn get_bind_group_layout(device: &Device) -> BindGroupLayout
    where
        Self: Sized;
    fn get_bind_group(&self) -> &BindGroup;
}

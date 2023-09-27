use core::fmt::{Debug, Formatter, Result};

use wgpu::{BindGroup, BindGroupLayout};

use crate::engine::{DiffuseTexture, LogicalDevice};

pub trait TMaterial {
    fn get_name(&self) -> &str;
    fn get_diffuse_texture(&self) -> &DiffuseTexture;
    fn get_bind_group_layout(logical_device: &LogicalDevice) -> BindGroupLayout
    where
        Self: Sized;
    fn get_bind_group(&self) -> &BindGroup;
}

impl Debug for dyn TMaterial {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.write_fmt(format_args!("TMaterial: {}", self.get_name()))
    }
}

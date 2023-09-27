use core::fmt::{Debug, Formatter, Result};

use wgpu::{BindGroup, BindGroupLayout};

use crate::engine::{DiffuseTexture, LogicalDevice};

pub trait TMaterial {
    fn name(&self) -> &str;
    fn diffuse_texture(&self) -> &DiffuseTexture;
    fn bind_group_layout(logical_device: &LogicalDevice) -> BindGroupLayout
    where
        Self: Sized;
    fn bind_group(&self) -> &BindGroup;
}

impl Debug for dyn TMaterial {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.write_fmt(format_args!("TMaterial: {}", self.name()))
    }
}

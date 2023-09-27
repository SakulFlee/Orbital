use wgpu::{logical_device, BindGroupLayout};

use crate::{AppObject, Model};

pub struct Cube {
    model: Model,
}

impl Cube {
    pub fn new(
        logical_device: &LogicalDevice,
        bind_group_layout: &BindGroupLayout,
    ) -> Result<Self, String> {
        Ok(Self {
            model: Model::from_path("cube/cube.obj", logical_device, bind_group_layout)?,
        })
    }
}

impl AppObject for Cube {
    fn model(&self) -> Option<&Model> {
        Some(&self.model)
    }

    fn do_render(&self) -> bool {
        true
    }
}

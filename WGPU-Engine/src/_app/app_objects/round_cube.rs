use wgpu::{logical_device, BindGroupLayout};

use crate::{AppObject, Model};

pub struct RoundCube {
    model: Model,
}

impl RoundCube {
    pub fn new(
        logical_device: &LogicalDevice,
        bind_group_layout: &BindGroupLayout,
    ) -> Result<Self, String> {
        Ok(Self {
            model: Model::from_path("cube/round_cube.obj", logical_device, bind_group_layout)?,
        })
    }
}

impl AppObject for RoundCube {
    fn model(&self) -> Option<&Model> {
        Some(&self.model)
    }

    fn do_render(&self) -> bool {
        true
    }
}

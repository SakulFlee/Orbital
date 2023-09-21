use wgpu::{BindGroupLayout, Device, Queue};

use crate::{AppObject, Model};

pub struct RoundCube {
    model: Model,
}

impl RoundCube {
    pub fn new(
        device: &Device,
        queue: &Queue,
        bind_group_layout: &BindGroupLayout,
    ) -> Result<Self, String> {
        Ok(Self {
            model: Model::from_path("cube/round_cube.obj", device, queue, bind_group_layout)?,
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

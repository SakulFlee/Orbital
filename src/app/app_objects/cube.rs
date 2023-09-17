use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroupLayout, BufferUsages, Device, Queue,
};

use crate::{AppObject, Mesh, Model, ModelVertex};

pub struct Cube {
    model: Model,
}

impl Cube {
    pub fn new(
        device: &Device,
        queue: &Queue,
        bind_group_layout: &BindGroupLayout,
    ) -> Result<Self, String> {
        Ok(Self {
            model: Model::from_path("cube/cube.obj", device, queue, bind_group_layout)?,
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

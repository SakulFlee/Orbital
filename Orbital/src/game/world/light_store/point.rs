use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutEntry, BindingType, Buffer,
    BufferBindingType, BufferDescriptor, BufferUsages, Device, Queue, ShaderStages,
};

use crate::resources::{descriptors::PipelineBindGroupLayout, realizations::PointLight};

#[derive(Debug)]
pub struct PointLightStore {
    dummy_added: bool,
    descriptors: Vec<PointLight>,
    bind_group: Option<BindGroup>,
    buffer: Option<Buffer>,
}

impl PointLightStore {
    pub fn new() -> Self {
        Self {
            dummy_added: true,
            descriptors: vec![PointLight::dummy()],
            bind_group: None,
            buffer: None,
        }
    }

    pub fn pipeline_bind_group_layout() -> PipelineBindGroupLayout {
        PipelineBindGroupLayout {
            label: "WorldEnvironment",
            entries: vec![BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        }
    }

    pub fn add_point_light(&mut self, point_light: PointLight) {
        // If dummy light is added, remove it first.
        if self.dummy_added {
            self.dummy_added = false;
            self.descriptors.clear();
        }

        self.descriptors.push(point_light);

        self.bind_group = None;
        self.buffer = None;
    }

    pub fn remove_point_light(&mut self, label: &str) {
        self.descriptors.retain(|x| x.label != label);

        self.bind_group = None;
        self.buffer = None;

        // If there are no more point lights, add the dummy light.
        if self.descriptors.is_empty() {
            self.dummy_added = true;
            self.descriptors.push(PointLight::dummy());
        }
    }

    pub fn bind_group(&self) -> &BindGroup {
        self.bind_group
            .as_ref()
            .expect("Must be initialized first!")
    }

    pub fn buffer(&self) -> &Buffer {
        self.buffer.as_ref().expect("Must be initialized first!")
    }

    pub fn update_buffer_and_bind_group(&mut self, device: &Device, queue: &Queue) {
        // Size of all the descriptors
        let size = (
            // Point Light size, times the amount of point lights
            // (padding is included)
            PointLight::bytes_needed() * self.descriptors.len()
        ) as u64;

        // Convert Point Lights into binary
        let data = self
            .descriptors
            .iter()
            .map(|x| x.to_bytes())
            .collect::<Vec<_>>()
            .concat();

        self.buffer = Some(device.create_buffer(&BufferDescriptor {
            label: Some("Point Light Storage Buffer"),
            size,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));

        queue.write_buffer(self.buffer.as_ref().unwrap(), 0, &data);

        let bind_group_layout = Self::pipeline_bind_group_layout().make_bind_group_layout(device);

        self.bind_group = Some(device.create_bind_group(&BindGroupDescriptor {
            label: Some("Light Storage Bind Group"),
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: self.buffer.as_ref().unwrap().as_entire_binding(),
            }],
        }));
    }

    pub fn update_if_needed(&mut self, device: &Device, queue: &Queue) {
        if self.bind_group.is_none() || self.buffer.is_none() {
            self.update_buffer_and_bind_group(device, queue);
        }
    }

    pub fn clear(&mut self) {
        self.descriptors.clear();

        // Re-add dummy light
        self.dummy_added = true;
        self.descriptors.push(PointLight::dummy());
    }
}

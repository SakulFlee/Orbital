use cgmath::{Vector3, Zero};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutEntry, BindingType, Buffer,
    BufferBindingType, BufferDescriptor, BufferUsages, Device, Queue, ShaderStages,
};

use crate::resources::descriptors::{LightDescriptor, PipelineBindGroupLayout};

#[derive(Debug)]
pub struct LightStorage {
    descriptors: Vec<LightDescriptor>,
    dummy_light_added: bool,
    needs_update: bool,
    bind_group: Option<BindGroup>,
    buffer: Option<Buffer>,
}

impl LightStorage {
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

    pub fn initialize(device: &Device, queue: &Queue) -> Self {
        let descriptors = vec![LightDescriptor::PointLight {
            position: Vector3::zero(),
            color: Vector3::zero(),
        }];

        let mut s = Self {
            descriptors,
            dummy_light_added: true,
            needs_update: false,
            bind_group: None,
            buffer: None,
        };

        s.update_buffer(device, queue);
        s.update_bind_group(device);

        s
    }

    pub fn clear(&mut self) {
        self.descriptors.clear();
        self.bind_group = None;
        self.buffer = None;
        self.dummy_light_added = false;
        self.needs_update = false;
    }

    pub fn update_buffer(&mut self, device: &Device, queue: &Queue) {
        if self.is_empty() {
            self.buffer = None;
            return;
        }

        // Size of all the descriptors
        let size = self.descriptors.iter().map(|x| x.bytes_needed()).sum();

        self.buffer = Some(device.create_buffer(&BufferDescriptor {
            label: Some("Light Storage Buffer"),
            size,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));

        let data = self
            .descriptors
            .iter()
            .map(|x| x.to_bytes())
            .collect::<Vec<_>>()
            .concat();

        queue.write_buffer(self.buffer.as_ref().unwrap(), 0, &data);
    }

    pub fn update_bind_group(&mut self, device: &Device) {
        if self.is_empty() {
            self.bind_group = None;
            return;
        }

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
        if !self.needs_update {
            return;
        }
        self.needs_update = false;

        if self.descriptors.is_empty() {
            self.buffer = None;
            self.bind_group = None;
        } else {
            self.update_buffer(device, queue);
            self.update_bind_group(device);
        }
    }

    pub fn add_descriptor(&mut self, descriptor: LightDescriptor) {
        if self.dummy_light_added {
            self.descriptors.remove(0);
            self.dummy_light_added = false;
        }

        self.descriptors.push(descriptor);
        self.needs_update = true;
    }

    pub fn is_empty(&self) -> bool {
        self.descriptors.is_empty()
    }

    pub fn bind_group(&self) -> Option<&BindGroup> {
        self.bind_group.as_ref()
    }
}

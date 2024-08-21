use wgpu::{BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, Device};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PipelineBindGroupLayout {
    pub label: &'static str,
    pub entries: Vec<BindGroupLayoutEntry>,
}

impl PipelineBindGroupLayout {
    pub fn make_bind_group_layout(&self, device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some(&self.label),
            entries: &self.entries,
        })
    }
}

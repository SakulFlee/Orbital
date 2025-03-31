use wgpu::{BindGroup, RenderPass};

mod resource;
pub use resource::*;

pub struct ShaderDescriptor {
    pub source: &'static str,
    pub resource_groups: Vec<ShaderResource>,
}

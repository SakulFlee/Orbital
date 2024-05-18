use wgpu::BindGroupDescriptor;

use super::{PipelineDescriptor, ShaderDescriptor, TextureDescriptor};

#[derive(Debug, Clone)]
pub enum MaterialDescriptor {
    PBR(TextureDescriptor),
    PBRCustomShader(TextureDescriptor, ShaderDescriptor),
    NoImports,
    Custom(BindGroupDescriptor<'static>, PipelineDescriptor),
}

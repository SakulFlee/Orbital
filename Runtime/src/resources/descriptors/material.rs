use super::{PipelineDescriptor, ShaderDescriptor, TextureDescriptor};

#[derive(Debug, Clone)]
pub enum MaterialDescriptor {
    PBR(TextureDescriptor),
    PBRCustomShader(TextureDescriptor, ShaderDescriptor),
    Custom(
        wgpu::BindGroupDescriptor<'static>,
        wgpu::BindGroupLayoutDescriptor<'static>,
        PipelineDescriptor,
    ),
}

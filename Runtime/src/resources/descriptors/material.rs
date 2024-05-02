use wgpu::{BindGroupDescriptor, BindGroupLayoutDescriptor};

use super::{PipelineDescriptor, ShaderDescriptor, TextureDescriptor};

#[derive(Debug)]
pub enum MaterialDescriptor<'a> {
    PBR(TextureDescriptor<'a>),
    PBRCustomShader(TextureDescriptor<'a>, ShaderDescriptor),
    Custom(
        BindGroupDescriptor<'a>,
        BindGroupLayoutDescriptor<'a>,
        PipelineDescriptor<'a>,
    ),
}

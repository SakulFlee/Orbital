use wgpu::TextureSampleType;

use crate::resources::{BufferDescriptor, TextureDescriptor};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum VariableType {
    Buffer(BufferDescriptor),
    Texture {
        descriptor: TextureDescriptor,
        sampler_type: TextureSampleType,
    },
    // TODO: BindingType::StorageTexture
}

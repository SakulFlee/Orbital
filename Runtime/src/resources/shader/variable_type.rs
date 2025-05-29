use wgpu::TextureSampleType;

use crate::resources::{BufferDescriptor, TextureDescriptor};

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum VariableType {
    Buffer(BufferDescriptor),
    Texture {
        descriptor: TextureDescriptor,
        sample_type: TextureSampleType,
    },
}

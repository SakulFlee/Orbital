use buffer::BufferDescriptor;
use texture::TextureDescriptor;
use wgpu::TextureSampleType;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum VariableType {
    Buffer(BufferDescriptor),
    Texture {
        descriptor: TextureDescriptor,
        sampler_type: TextureSampleType,
    },
    // TODO: BindingType::StorageTexture
}

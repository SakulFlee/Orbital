use buffer::BufferDescriptor;
use texture::TextureDescriptor;
use wgpu::TextureSampleType;

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum VariableType {
    Buffer(BufferDescriptor),
    Texture {
        descriptor: TextureDescriptor,
        sampler_type: TextureSampleType,
    },
    // TODO: BindingType::StorageTexture
}

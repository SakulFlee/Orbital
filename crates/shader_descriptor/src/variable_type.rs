use buffer_descriptor::BufferDescriptor;
use texture_descriptor::TextureDescriptor;
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

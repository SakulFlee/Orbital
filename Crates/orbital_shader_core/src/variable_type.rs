use orbital_texture::{BufferDescriptor, TextureDescriptor};
use wgpu::{SamplerBindingType, TextureSampleType};

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum VariableType {
    Buffer(BufferDescriptor),
    Texture {
        descriptor: Box<TextureDescriptor>,
        sample_type: TextureSampleType,
        sampler_binding_type: SamplerBindingType,
    },
}

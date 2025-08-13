use wgpu::{SamplerBindingType, TextureSampleType};

use crate::resources::{BufferDescriptor, TextureDescriptor};

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum VariableType {
    Buffer(BufferDescriptor),
    Texture {
        descriptor: TextureDescriptor,
        sample_type: TextureSampleType,
        sampler_binding_type: SamplerBindingType,
    },
    Shared {
        label: String,
    },
}

impl VariableType {
    /// Wrapper for [`Self::Shared`]
    #[allow(non_snake_case)]
    pub fn GlobalVariable(label: String) -> Self {
        Self::Shared { label }
    }
}

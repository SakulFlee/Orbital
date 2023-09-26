use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct AmbientLightUniform {
    /// Color of the ambient light
    color: [f32; 3],

    /// Strength of the ambient light
    strength: f32,
}

impl AmbientLightUniform {
    pub fn empty() -> Self {
        Self::new([0.0, 0.0, 0.0], 0.0)
    }

    pub fn new(color: [f32; 3], strength: f32) -> Self {
        Self { color, strength }
    }
}

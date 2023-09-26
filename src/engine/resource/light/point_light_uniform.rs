use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct PointLightUniform {
    /// Color of the point light
    color: [f32; 3],

    /// Position of the point light
    position: [f32; 3],

    /// Strength of the point light
    strength: f32,

    /// Whether the light is enabled or not
    /// 0 = false == disabled
    /// 1 = true  == enabled
    /// Note: This type is a `u32` instead of a `bool` to be
    /// compliant with the Uniform memory layout of
    /// being 2^x in size.
    enabled: u32,
}

impl PointLightUniform {
    pub fn new(color: [f32; 3], position: [f32; 3], strength: f32, enabled: bool) -> Self {
        Self {
            color,
            position,
            strength,
            enabled: if enabled { 1 } else { 0 },
        }
    }
}

use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct PointLightUniform {
    /// Color of the point light
    ///
    /// Vec4 to avoid spacing and padding issues!
    /// Since `[f32; 3]` would be 12 bytes, but WGPU bumps it up to 16,
    /// a spacer/padding would be required. Vec4 acts as a spacer here
    /// for us, without extra fields.
    color: [f32; 4],

    /// Position of the point light
    ///
    /// Vec4 to avoid spacing and padding issues!
    /// Since `[f32; 3]` would be 12 bytes, but WGPU bumps it up to 16,
    /// a spacer/padding would be required. Vec4 acts as a spacer here
    /// for us, without extra fields.
    position: [f32; 4],

    /// Strength of the point light
    strength: f32,

    /// Whether the light is enabled or not
    /// 0 = false == disabled
    /// 1 = true  == enabled
    /// Note: This type is a `u32` instead of a `bool` to be
    /// compliant with the Uniform memory layout of
    /// being 2^x in size.
    enabled: u32,

    /// Padding
    ///
    /// The size of the above fields result in:
    /// vec4 -> 16 bytes
    /// f32/u32 -> 4 bytes
    ///
    /// We have two vec4's and two f32/u32's.
    /// Thus: 2x16 + 2x4 == 40 bytes are used.
    ///
    /// However, WGPU/WGSL expects us to have a buffer of the size of
    /// a power of 2 (2^x).
    /// The next closest value is 48.
    ///
    /// 48 - 40 bytes used == 8 bytes need to be added as a "spacer"/"padding".
    /// 2x u32 equal those 8 bytes
    _padding: [u32; 2],
}

impl PointLightUniform {
    pub fn new(color: [f32; 3], position: [f32; 3], strength: f32, enabled: bool) -> Self {
        Self {
            color: [color[0], color[1], color[2], 0.0],
            position: [position[0], position[1], position[2], 0.0],
            strength,
            enabled: if enabled { 1 } else { 0 },
            _padding: [0, 0],
        }
    }

    pub fn empty() -> Self {
        Self::new((0.0, 0.0, 0.0).into(), (0.0, 0.0, 0.0).into(), 0.0, false)
    }
}

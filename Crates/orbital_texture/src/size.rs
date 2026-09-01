#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureSize {
    pub width: u32,
    pub height: u32,
    pub depth_or_array_layers: u32,
    pub base_mip: u32,
    pub mip_levels: u32,
}

impl Default for TextureSize {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            depth_or_array_layers: 1,
            base_mip: 0,
            mip_levels: 1,
        }
    }
}

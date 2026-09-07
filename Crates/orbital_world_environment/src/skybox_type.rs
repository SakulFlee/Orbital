#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SkyboxType {
    Diffuse,
    Specular { lod: u8 },
}

impl Default for SkyboxType {
    fn default() -> Self {
        Self::Specular { lod: 0 }
    }
}

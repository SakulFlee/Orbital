pub type ShaderSource = &'static str;

pub const STANDARD_VERTEX_SHADER: &'static str = include_str!("standard.vert");
pub const STANDARD_FRAGMENT_SHADER: &'static str = include_str!("standard.frag");
pub const SHADER_UTIL_GENERAL: &'static str = include_str!("util.glsl");
pub const SHADER_UTIL_PBR: &'static str = include_str!("pbr.glsl");

#[derive(Debug, Clone)]
pub struct ShaderDescriptor {
    pub identifier: String,
    pub vertex_source: ShaderSource,
    pub fragment_source: ShaderSource,
    pub includes: Vec<ShaderSource>,
}

impl Default for ShaderDescriptor {
    fn default() -> Self {
        Self {
            identifier: "standard_pbr".into(),
            vertex_source: STANDARD_VERTEX_SHADER,
            fragment_source: STANDARD_FRAGMENT_SHADER,
            includes: vec![SHADER_UTIL_GENERAL, SHADER_UTIL_PBR],
        }
    }
}

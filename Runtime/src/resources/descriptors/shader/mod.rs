#[derive(Debug, Clone)]
pub enum ShaderSource {
    FromFile(&'static str),
    FromSourceString(&'static str),
}

#[derive(Debug, Clone)]
pub struct ShaderDescriptor {
    pub identifier: String,
    pub source: ShaderSource,
}

impl Default for ShaderDescriptor {
    fn default() -> Self {
        Self {
            identifier: "standard_pbr".into(),
            source: ShaderSource::FromSourceString(include_str!("standard_pbr.wgsl")),
        }
    }
}

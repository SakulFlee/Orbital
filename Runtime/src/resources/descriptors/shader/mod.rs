#[derive(Debug)]
pub enum ShaderSource {
    FromFile(&'static str),
    FromSourceString(&'static str),
}

#[derive(Debug)]
pub struct ShaderDescriptor {
    pub identifier: &'static str,
    pub source: ShaderSource,
}

impl Default for ShaderDescriptor {
    fn default() -> Self {
        Self {
            identifier: "standard_pbr",
            source: ShaderSource::FromSourceString(include_str!("standard_pbr.wgsl")),
        }
    }
}

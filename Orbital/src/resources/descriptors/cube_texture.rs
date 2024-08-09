use cgmath::Vector2;

#[derive(Debug)]
pub enum CubeTextureDescriptor {
    RadianceHDRFile { path: &'static str },
    RadianceHDRData { data: Vec<u8>, size: Vector2<u32> },
}

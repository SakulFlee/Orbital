use cgmath::Vector2;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CubeTextureDescriptor {
    /// Loading an HDR from a file.  
    /// Will convert the HDR equirectangular image into a CubeMap.
    /// 
    /// Use this over [CubeTextureDescriptor::RadianceHDRData] if possible.
    RadianceHDRFile {
        cube_face_size: u32,
        path: &'static str,
    },
    /// For directly loading an HDR from bytes.  
    /// Only use if you really need to as
    /// [CubeTextureDescriptor::RadianceHDRFile] does some additional
    /// magic like adding Alpha channels.
    ///
    /// The bytes here are required to be RGBA (Alpha channel is a must!) and be
    /// 32 bit floats (f32 == 4x bytes).
    RadianceHDRData {
        cube_face_size: u32,
        data: Vec<u8>,
        size: Vector2<u32>,
    },
}

impl CubeTextureDescriptor {
    pub const SKY_BOX_DEFAULT_SIZE: u32 = 1024;
}

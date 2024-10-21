use cgmath::Vector2;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WorldEnvironmentDescriptor {
    /// Loading an HDRI from file.  
    /// First of all, will convert the HDRI _equirectangular_ image
    /// into a _cube texture_.
    /// Secondly, will transform the _cube texture_ into a diffuse
    /// (irradiance) and specular (radiance) _cube texture_.
    FromFile {
        cube_face_size: u32,
        path: &'static str,
    },
    /// Same as [WorldEnvironmentDescriptor::FromFile], but uses a data
    /// Vector instead.
    ///
    /// ⚠️ Make sure the data you supply is correct and contains an
    /// alpha channel!
    FromData {
        cube_face_size: u32,
        data: Vec<u8>,
        size: Vector2<u32>,
    },
}

impl WorldEnvironmentDescriptor {
    pub const SKY_BOX_DEFAULT_SIZE: u32 = 1024;
}

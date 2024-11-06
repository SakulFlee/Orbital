use cgmath::{Vector2, Vector4};

#[derive(Debug, Clone, Hash)]
pub enum TextureDescriptor {
    /// Creates a texture by loading it from a file.
    /// ⚠️ This file must be accessible during runtime!
    ///
    /// For supported formats check the [Image documentation](https://github.com/image-rs/image/blob/main/README.md#supported-image-formats).
    FilePath(&'static str),
    /// Creates a standard SRGB texture from bytes (`u8`).
    ///
    /// # Parameters
    /// 1st.: A Vector of bytes, containing the image data in RGBA pixels
    /// 2nd.: The size of the texture, must be accurate or this will panic!
    StandardSRGBAu8Data(Vec<u8>, Vector2<u32>),
    /// Creates a texture with a single uniform color.
    ///
    /// The format is:  
    /// x -> Red  
    /// y -> Green  
    /// z -> Blue  
    /// w -> Alpha  
    ///
    /// Each number should be at most 255.
    /// Where 255 means 100% and 0 means 0% of that channel.
    ///
    /// Each colour channel should be converted internally to a float via:
    ///
    /// ```ignore
    /// # use orbital::cgmath::Vector4;
    ///
    /// let channel_{x, y, z, w}: f64 = Vector4::{x, y, z, w} / 255.0f64;
    /// ```
    ///
    /// # Examples
    ///
    /// 100% Red:
    /// ```ignore
    /// # use orbital::cgmath::Vector4;
    ///
    /// Vector4 {
    ///     x: 255,
    ///     y: 0,
    ///     z: 0,
    ///     w: 255
    /// }
    /// ```
    ///
    /// 33.3% Red, 33.3% Blue, 33.3% Blue:
    /// ```ignore
    /// # use orbital::cgmath::Vector4;
    ///
    /// Vector4 {
    ///     x: 85,
    ///     y: 85,
    ///     z: 85,
    ///     w: 255
    /// }
    /// ```
    UniformColor(Vector4<u8>),
    /// Grayscale/Single channel textures
    Luma { data: Vec<u8>, size: Vector2<u32> },
    /// Grayscale/Single channel textures
    UniformLuma { data: u8 },
    /// Creates a depth texture
    Depth(Vector2<u32>),
}

impl TextureDescriptor {
    pub const EMPTY: Self = Self::UNIFORM_BLACK;
    pub const UNIFORM_BLACK: Self = Self::UniformColor(Vector4 {
        x: 0,
        y: 0,
        z: 0,
        w: 255,
    });
    pub const UNIFORM_WHITE: Self = Self::UniformColor(Vector4 {
        x: 255,
        y: 255,
        z: 255,
        w: 255,
    });
    pub const UNIFORM_GRAY: Self = Self::UniformColor(Vector4 {
        x: 128,
        y: 128,
        z: 128,
        w: 255,
    });
    pub const UNIFORM_LUMA_BLACK: Self = Self::UniformLuma { data: 0 };
    pub const UNIFORM_LUMA_WHITE: Self = Self::UniformLuma { data: 255 };
    pub const UNIFORM_LUMA_GRAY: Self = Self::UniformLuma { data: 128 };
}

impl PartialEq for TextureDescriptor {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::FilePath(l0), Self::FilePath(r0)) => l0 == r0,
            (Self::StandardSRGBAu8Data(l0, l1), Self::StandardSRGBAu8Data(r0, r1)) => {
                // Note: This should be much faster than just doing `l0 == r0 && l1 == r1` as this **fails fast**!
                // We first compare the obvious facts, like the size of the data as well as the resolution of the texture.
                // If those actually are equal, which already might be rare, we compare byte-by-byte but in an iterator and with fail fast enabled (any).
                // Thus, as soon as a single byte is not equal, we fail the equality check and saving a lot of time this way!
                if l1 != r1 || l0.len() != r0.len() {
                    return false;
                }

                l0.iter().zip(r0.iter()).any(|(l, r)| l.eq(r))
            }
            (Self::UniformColor(l0), Self::UniformColor(r0)) => l0 == r0,
            (
                Self::Luma {
                    data: l_data,
                    size: l_size,
                },
                Self::Luma {
                    data: r_data,
                    size: r_size,
                },
            ) => l_data == r_data && l_size == r_size,
            (Self::UniformLuma { data: l_data }, Self::UniformLuma { data: r_data }) => {
                l_data == r_data
            }
            (Self::Depth(l0), Self::Depth(r0)) => l0 == r0,
            _ => false,
        }
    }
}

impl Eq for TextureDescriptor {}

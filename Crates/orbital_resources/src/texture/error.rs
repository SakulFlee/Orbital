#[derive(Debug)]
pub enum TextureError {
    ImageError(image::ImageError),
    IOError(std::io::Error),
    DataSizeMismatch {
        expected: usize,
        actual: usize,
        width: u32,
        height: u32,
        bytes_per_pixel: u32,
    },
}

impl std::fmt::Display for TextureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ImageError(e) => write!(f, "Image error: {e}"),
            Self::IOError(e) => write!(f, "IO error: {e}"),
            Self::DataSizeMismatch {
                expected,
                actual,
                width,
                height,
                bytes_per_pixel,
            } => write!(
                f,
                "Texture data size mismatch: expected {expected} bytes ({width}x{height}x{bytes_per_pixel}), got {actual} bytes"
            ),
        }
    }
}

impl std::error::Error for TextureError {}

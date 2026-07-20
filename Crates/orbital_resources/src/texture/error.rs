#[derive(Debug)]
pub enum TextureError {
    ImageError(image::ImageError),
    IOError(std::io::Error),
}

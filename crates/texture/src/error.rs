#[derive(Debug)]
pub enum Error {
    ImageError(image::ImageError),
    IOError(std::io::Error),
}

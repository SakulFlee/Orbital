use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),
    Image(image::ImageError),
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

use std::fmt::Display;

#[derive(Debug)]
pub enum WorldEnvironmentError {
    IO(std::io::Error),
    Image(image::ImageError),
}

impl std::error::Error for WorldEnvironmentError {}

impl Display for WorldEnvironmentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

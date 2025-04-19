#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),
    Image(image::ImageError),
}

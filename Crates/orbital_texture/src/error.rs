#[derive(Debug)]
pub enum TextureError {
    ImageError(image::ImageError),
    IOError(std::io::Error),
    FsError(orbital_file_manager::FsError),
}

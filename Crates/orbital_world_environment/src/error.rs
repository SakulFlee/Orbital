use std::fmt::Display;

#[derive(Debug)]
pub enum WorldEnvironmentError {
    IO(std::io::Error),
    Image(image::ImageError),
    Fs(orbital_file_manager::FsError),
    Msg(String),
}

impl WorldEnvironmentError {
    pub fn msg(msg: impl Into<String>) -> Self {
        Self::Msg(msg.into())
    }
}

impl std::error::Error for WorldEnvironmentError {}

impl Display for WorldEnvironmentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IO(e) => write!(f, "IO error: {e}"),
            Self::Image(e) => write!(f, "Image error: {e}"),
            Self::Fs(e) => write!(f, "FileManager error: {e}"),
            Self::Msg(m) => write!(f, "{m}"),
        }
    }
}

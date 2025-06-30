use std::error::Error;
use std::fmt::{Display, Formatter};
use gltf::Node;

#[derive(Debug)]
pub enum GltfError {
    /// Thrown if a given operation is unsupported
    Unsupported
}

impl Display for GltfError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GltfError::Unsupported => write!(f, "Unsupported!"),
        }
    }
}

impl Error for GltfError {}

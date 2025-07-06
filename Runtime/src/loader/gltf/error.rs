use crate::loader::gltf::SpecificGltfImport;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum GltfError {
    /// Thrown if a given operation is unsupported
    Unsupported,
    NotFound(SpecificGltfImport),
}

impl Display for GltfError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GltfError::Unsupported => write!(f, "Unsupported!"),
            GltfError::NotFound(import) => {
                write!(f, "Couldn't find specific glTF import: {import:?}")
            }
        }
    }
}

impl Error for GltfError {}

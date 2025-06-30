use std::error::Error;
use std::fmt::{Display, Formatter};
use gltf::Node;

#[derive(Debug)]
pub enum GltfError {
    /// Thrown if a given node is supposed to be parsed as _Mesh_, but isn't of type _Mesh_.
    NodeNotMesh,
    /// Thrown if a given node is supposed to be parsed as _Camera_, but isn't of type _Camera_.
    NodeNotCamera,
}

impl Display for GltfError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GltfError::NodeNotMesh => write!(f, "Node is not of type Mesh!"),
            GltfError::NodeNotCamera => write!(f, "Node is not of type Camera!"),
        }
    }
}

impl Error for GltfError {}

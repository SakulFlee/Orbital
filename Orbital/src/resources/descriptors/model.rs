use crate::transform::Transform;

use super::{MaterialDescriptor, MeshDescriptor};

/// Descriptor for a model
///
/// TODO
#[derive(Debug, Clone)]
pub struct ModelDescriptor {
    pub label: String,
    pub mesh: MeshDescriptor,
    pub material: MaterialDescriptor,
    pub transforms: Vec<Transform>,
}

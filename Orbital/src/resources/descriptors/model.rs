use super::{Instancing, MaterialDescriptor, MeshDescriptor};

/// Descriptor for a model
///
/// Instancing is used to optimize draw calls.
/// Use [Instancing::Single] to describe a **single** model.
/// Use [Instancing::Multiple] to describe **multiple** instances
/// **of the same model**.
///
/// Utilizing [Instancing::Multiple], compared to [Instancing::Single],
/// saves draw calls, makes your render faster and more optimized and saves
/// resources.
/// However, this is **only** possible for the exact same model!
/// Some parameters, like position, rotation and scale, can be altered via
/// instancing. However, you can't instance a different model or material.
#[derive(Debug, Clone)]
pub enum ModelDescriptor {
    /// Describes a model to be created from a mesh and a material descriptor.
    FromDescriptors {
        label: String,
        mesh: MeshDescriptor,
        material: MaterialDescriptor,
        instancing: Instancing,
    },
}

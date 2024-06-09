use super::{ImportDescriptor, Instancing, MaterialDescriptor, MeshDescriptor};

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
    ///
    /// # Arguments
    /// 
    /// * 1.: Mesh descriptor, defines the [Mesh](crate::resources::realizations::Mesh) of a [Model]
    /// 
    /// * 2.: Material descriptor, defines the [Material](crate::resources::realizations::Material) of a [Model]
    /// 
    /// * 3.: Instancing  
    ///     Check super description for [Instancing] explanation.
    FromDescriptors(MeshDescriptor, MaterialDescriptor, Instancing),
    /// Describes a model to be imported from a _glTF file_.
    /// 
    /// Note, that this **only** imports a [Model](crate::resources::realizations::Model) (i.e. a [Mesh](crate::resources::realizations::Mesh) + a [Material](crate::resources::realizations::Material)).
    /// You can also import as a [Composition](crate::resources::realizations::Composition) (i.e. [Model](crate::resources::realizations::Model) + [Light](crate::resources::realizations::Light), [Camera](crate::resources::realizations::Camera), etc.) via a [CompositionDescriptor](crate::resources::descriptors::CompositionDescriptor).
    /// 
    /// # Arguments
    /// 
    /// * 1.: Path to the _glTF File_.  
    ///     ⚠️ The file must be accessible at runtime.
    ///
    /// * 2. &
    ///   3.: Control what is imported.  
    ///     A single _glTF file_ may contain multiple _Models_.
    ///     Therefore, the 2nd parameter defines which _Scene_ the _Model_
    ///     should be loaded from.
    ///     The 3rd parameter defines which Model is being imported.
    ///     Check [ImportDescriptor] for more!
    ///
    /// * 4.: Instancing.
    ///     Check super description for [Instancing] explanation.
    #[cfg(feature = "gltf")]
    FromGLTF(&'static str, ImportDescriptor, ImportDescriptor, Instancing),
}

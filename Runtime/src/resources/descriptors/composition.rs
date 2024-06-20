use super::{ImportDescriptor, ModelDescriptor};

#[derive(Debug)]
pub enum CompositionDescriptor {
    /// Describes a composition made out of a vector of [ModelDescriptor]s.
    FromDescriptors(Vec<ModelDescriptor>),
    /// Describes a model to be imported from a _glTF file_.
    ///
    /// Note, that this imports **all** data from the glTF (i.e. [Model](crate::resources::realizations::Model) + [Light](crate::resources::realizations::Light), [Camera](crate::resources::realizations::Camera), etc.).
    /// You can also import **only** the [Mesh](crate::resources::realizations::Mesh) and [Material](crate::resources::realizations::Material) as a [Model](crate::resources::realizations::Model).
    /// Check [MaterialDescriptor] for more.
    ///
    /// # Arguments
    ///
    /// 1.: Path to the _glTF File_.  
    ///     ⚠️ The file must be accessible at runtime.
    ///
    /// 2.: Control what is imported.  
    ///     A single _glTF file_ may contain multiple _glTF Scenes_.
    ///     Therefore, the 2nd parameter defines which _glTF Scene_
    ///     should be imported.
    #[cfg(feature = "gltf")]
    FromGLTF(&'static str, ImportDescriptor),
}

pub type SceneDescriptor = CompositionDescriptor;

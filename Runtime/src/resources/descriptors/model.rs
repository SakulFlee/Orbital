use super::{ImportDescriptor, MaterialDescriptor, MeshDescriptor};

#[derive(Debug, Clone)]
pub enum ModelDescriptor {
    /// Describes a model to be created from a mesh and a material descriptor.
    FromDescriptors(MeshDescriptor, MaterialDescriptor),
    /// Describes a model to be imported from a _glTF file_.
    ///
    /// The first parameter is the path to the file.
    /// ⚠️ The file must be accessible at runtime.
    /// ⚠️ This isn't necessarily handled automatically!
    /// TODO: Add an asset inclusion system for builds
    ///
    /// The second and third parameter control what is imported.
    /// A single _glTF file_ may contain multiple _Models_.
    /// Therefore, the 2nd parameter defines which _Scene_ the _Model_
    /// should be loaded from.
    /// The 3rd parameter defines which Model is being imported.
    /// Check [`ImportDescriptor`] for more!
    ///
    /// Note, that this **only** imports a _Model_ (i.e. a _Mesh_ +
    /// a _Material_).
    /// You can also import as a _Composition/Scene_ (i.e. _Models_ +
    /// _Lights_, _Camera_, etc.) via a [`SceneDescriptor`].
    #[cfg(feature = "gltf")]
    FromGLTF(&'static str, ImportDescriptor, ImportDescriptor),
}

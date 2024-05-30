use super::{MaterialDescriptor, MeshDescriptor};

#[derive(Debug, Clone)]
pub enum ModelDescriptor {
    /// Creates a model descriptor from existing mesh and material descriptors
    FromDescriptors(MeshDescriptor, MaterialDescriptor),
    /// A full path to the file you want to load.
    /// This file path must be accessible at runtime.
    /// Note, that Rust doesn't package files by default.
    /// You will need a way to package any assets with your
    /// application!
    /// 
    /// Note, that a file can potentially contain multiple models.
    /// In such cases, multiple models will be returned on realization.
    FromFile(&'static str),
    /// Create a model (mesh + material) from bytes and a hint.
    ///
    /// The bytes need to be in a format supported by ASSIMP.
    /// A list of supported formats can be found here:
    /// https://assimp-docs.readthedocs.io/en/stable/about/introduction.html
    ///
    /// The hint is used by ASSIMP to determine which format our data is in.
    /// In most cases, this is the file extension of our file.
    /// For example, glTF files commonly are named "MyFile.gltf", the ending
    /// "gltf" part is the hint you want to enter!
    /// The same applies to basically any format. Some common ones are:
    /// glTF: gltf
    /// Wavefront Object: obj
    /// FBX: fbx
    /// 
    /// Note, that a file can potentially contain multiple models.
    /// In such cases, multiple models will be returned on realization.
    FromBytes(&'static [u8], &'static str),
}

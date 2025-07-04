use crate::loader::gltf::import_type::GltfImportType;
use crate::loader::gltf::SpecificGltfImport;

/// Used to define what is being imported from a glTF file.
///
/// Please note that labels in glTF is an _optional_ feature and _can be disabled_.
/// Some applications may have a specific setting to _enable label support_.
#[derive(Debug)]
pub enum GltfImport {
    /// To import the whole file.
    /// Note that this will import **all** scenes, but scenes aren't a concept of Orbital (yet?).
    /// Meaning, if you have e.g. multiple levels defined and sorted by scenes, you will import them
    /// all on-top of each other.
    /// However, caching or further changing the position of each imported resource might work!
    WholeFile,
    /// To import one or multiple specific "thing" from a glTF file.
    Specific(Vec<SpecificGltfImport>),
}

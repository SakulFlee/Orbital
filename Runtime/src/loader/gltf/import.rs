use crate::loader::gltf::import_type::GltfImportType;

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
    /// To import a whole scene from a glTF file.
    WholeScene {
        /// The label of the scene
        label: String,
    },
    /// To import a set of "things" from a glTF file.
    /// The vector needs to be filled with a tuple consisting of first,
    /// the import type you want to import, and second, the label of the "thing".
    Set {
      vec: Vec<(GltfImportType, String)>,  
    },
    /// To import a specific "thing" from a glTF file.
    Specific {
        /// The type of "thing" to import
        import_type: GltfImportType,
        /// The label of the "thing" to import
        label: String,
    },
}
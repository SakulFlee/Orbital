use crate::loader::gltf::GltfImport;

/// Defines how a given glTF file is being imported.
#[derive(Debug)]
pub struct GltfImportTask {
    pub file: String,
    pub import: GltfImport,
}

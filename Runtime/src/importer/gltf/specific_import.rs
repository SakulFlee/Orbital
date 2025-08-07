use crate::loader::gltf::GltfImportType;

#[derive(Debug)]
pub struct SpecificGltfImport {
    /// The type of "thing" to import
    pub import_type: GltfImportType,
    /// The label of the "thing" to import
    pub label: String,
}

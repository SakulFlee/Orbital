use crate::gltf::GltfImportType;

#[derive(Debug)]
pub struct SpecificGltfImport {
    pub import_type: GltfImportType,
    pub label: String,
}

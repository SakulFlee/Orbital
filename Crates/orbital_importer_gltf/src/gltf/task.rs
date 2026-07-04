use crate::gltf::GltfImport;

#[derive(Debug)]
pub struct GltfImportTask {
    pub file: String,
    pub import: GltfImport,
}

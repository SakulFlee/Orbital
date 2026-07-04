use crate::gltf::SpecificGltfImport;

#[derive(Debug)]
pub enum GltfImport {
    WholeFile,
    Specific(Vec<SpecificGltfImport>),
}

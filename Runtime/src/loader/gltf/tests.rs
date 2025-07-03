use async_std::task::block_on;
use log::debug;
use crate::loader::gltf::{GltfImport, GltfImportTask, GltfImportType, GltfImporter, SpecificGltfImport};
use crate::logging;

#[test]
fn test_load_camera() {
    logging::test_init();
    
    let task = GltfImportTask {
        file: "../Assets/Models/TestScene.gltf".to_string(),
        import: GltfImport::Specific(vec![SpecificGltfImport {
            label: "TestCamera".to_string(),
            import_type: GltfImportType::Camera
        }]),
    };
    
    let x = GltfImporter::import(task);
    let y = block_on(x);
    debug!("{:?}", y);
    assert!(y.is_ok());
}

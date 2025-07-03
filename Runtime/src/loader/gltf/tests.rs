use async_std::task::block_on;
use log::debug;
use crate::loader::gltf::{GltfImport, GltfImportTask, GltfImportType, GltfImporter, SpecificGltfImport};
use crate::logging;

#[test]
fn load_gltf() {
    logging::test_init();

    let task = GltfImportTask {
        file: "../Examples/SharedAssets/Models/TestScene.gltf".to_string(),
        import: GltfImport::WholeFile,
    };

    let x = GltfImporter::import(task);
    let result = block_on(x);
    debug!("{:?}", result);

    assert!(result.errors.is_empty());
}


#[test]
fn load_glb() {
    logging::test_init();

    let task = GltfImportTask {
        file: "../Examples/SharedAssets/Models/TestScene.glb".to_string(),
        import: GltfImport::WholeFile,
    };

    let x = GltfImporter::import(task);
    let result = block_on(x);
    debug!("{:?}", result);

    assert!(result.errors.is_empty());
}

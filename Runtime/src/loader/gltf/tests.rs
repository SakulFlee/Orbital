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
    let (_models, _cameras, errors) = block_on(x);
    debug!("{:?}", errors);
    assert!(errors.is_empty());
}


#[test]
fn load_glb() {
    logging::test_init();

    let task = GltfImportTask {
        file: "../Examples/SharedAssets/Models/TestScene.glb".to_string(),
        import: GltfImport::WholeFile,
    };

    let x = GltfImporter::import(task);
    let (_models, _cameras, errors) = block_on(x);
    debug!("{:?}", errors);
    assert!(errors.is_empty());
}

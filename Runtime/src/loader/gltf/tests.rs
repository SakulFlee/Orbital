use async_std::task::block_on;
use cgmath::Point3;
use log::debug;
use crate::loader::gltf::{GltfImport, GltfImportResult, GltfImportTask, GltfImportType, GltfImporter, SpecificGltfImport};
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

fn query(import: SpecificGltfImport) -> GltfImportResult {
    logging::test_init();

    let task = GltfImportTask {
        file: "../Examples/SharedAssets/Models/TestScene.gltf".to_string(),
        import: GltfImport::Specific(vec![import]),
    };

    let x = GltfImporter::import(task);
    let result = block_on(x);
    debug!("{:?}", result);
    assert!(result.errors.is_empty());

    result
}

#[test]
fn check_top_camera_existing() {
    let result = query(SpecificGltfImport {
        import_type: GltfImportType::Camera,
        label: "Top Camera".to_string(),
    });
    assert_eq!(result.cameras.len(), 1);
}

#[test]
fn check_default_camera_existing() {
    let result = query(SpecificGltfImport {
        import_type: GltfImportType::Camera,
        label: "Default Camera".to_string(),
    });
    assert_eq!(result.cameras.len(), 1);
}

#[test]
fn check_red_cube_existing() {
    let result = query(SpecificGltfImport {
        import_type: GltfImportType::Model,
        label: "Red Cube".to_string(),
    });
    assert_eq!(result.models.len(), 1);
}

#[test]
fn check_blue_cube_existing() {
    let result = query(SpecificGltfImport {
        import_type: GltfImportType::Model,
        label: "Blue Cube".to_string(),
    });
    assert_eq!(result.models.len(), 1);
}

#[test]
fn check_green_cube_existing() {
    let result = query(SpecificGltfImport {
        import_type: GltfImportType::Model,
        label: "Green Cube".to_string(),
    });
    assert_eq!(result.models.len(), 1);
}

#[test]
fn check_top_camera_position_matches() {
    let result = query(SpecificGltfImport {
        import_type: GltfImportType::Camera,
        label: "Top Camera".to_string(),
    });
    assert_eq!(result.cameras.len(), 1);

    let camera = &result.cameras[0];
    assert_eq!(camera.position, Point3::new(0.0, 5.0, 0.0));
}

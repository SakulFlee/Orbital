//! End-to-end tests for the glTF importer's cross-platform asset loading.
//!
//! Exercises the custom in-memory import path with a temp
//! [`FileManager`](orbital_file_manager::FileManager): a `.gltf` referencing
//! external `.bin`/`.png` files (the Android-relevant case) and a self-contained
//! `.glb` (the case the repo's examples use).

use std::io::Cursor;

use image::{ImageFormat, Rgba, RgbaImage};
use orbital_file_manager::{DesktopAssetSource, DirStorage, FileManager};
use orbital_importer_gltf::{GltfImport, GltfImportTask, GltfImporter};
use serde_json::json;

/// Creates a throwaway directory with an `Assets/Models/` layout and returns it.
fn temp_assets(tag: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!("orbital_gltf_test_{tag}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("Assets").join("Models")).expect("create temp assets");
    root
}

/// A [`FileManager`] rooted at a temp directory (assets under `Assets/`).
fn make_file_manager(root: &std::path::Path) -> FileManager {
    FileManager::new(
        Box::new(DesktopAssetSource::with_base_dir(root.join("Assets"))),
        Box::new(DirStorage::new(root.to_path_buf())),
    )
}

/// A 3-vertex triangle: positions (3×vec3f32), normals (3×vec3f32), uvs
/// (3×vec2f32), indices (3×u16) = 102 bytes, tightly packed.
fn mesh_bin() -> Vec<u8> {
    let mut bin = Vec::new();
    for point in [[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]] {
        for component in point {
            bin.extend_from_slice(&component.to_le_bytes());
        }
    }
    for _ in 0..3 {
        for component in [0.0f32, 0.0, 1.0] {
            bin.extend_from_slice(&component.to_le_bytes());
        }
    }
    for uv in [[0.0f32, 0.0], [1.0, 0.0], [0.0, 1.0]] {
        for component in uv {
            bin.extend_from_slice(&component.to_le_bytes());
        }
    }
    for index in [0u16, 1, 2] {
        bin.extend_from_slice(&index.to_le_bytes());
    }
    bin
}

fn base_json() -> serde_json::Value {
    json!({
        "asset": { "version": "2.0", "generator": "orbital_fm_test" },
        "scene": 0,
        "scenes": [ { "nodes": [0] } ],
        "nodes": [ { "mesh": 0, "name": "Triangle" } ],
        "meshes": [ { "primitives": [
            { "attributes": { "POSITION": 0, "NORMAL": 1, "TEXCOORD_0": 2 }, "indices": 3, "material": 0 }
        ] } ],
        "materials": [ { "name": "Base", "pbrMetallicRoughness": {
            "baseColorTexture": { "index": 0 },
            "metallicRoughnessTexture": { "index": 1 }
        } } ],
        "textures": [ { "source": 0 }, { "source": 1 } ],
        "accessors": [
            { "bufferView": 0, "componentType": 5126, "count": 3, "type": "VEC3", "min": [0.0, 0.0, 0.0], "max": [1.0, 1.0, 0.0] },
            { "bufferView": 1, "componentType": 5126, "count": 3, "type": "VEC3" },
            { "bufferView": 2, "componentType": 5126, "count": 3, "type": "VEC2" },
            { "bufferView": 3, "componentType": 5123, "count": 3, "type": "SCALAR" }
        ],
        "bufferViews": [
            { "buffer": 0, "byteOffset": 0, "byteLength": 36 },
            { "buffer": 0, "byteOffset": 36, "byteLength": 36 },
            { "buffer": 0, "byteOffset": 72, "byteLength": 24 },
            { "buffer": 0, "byteOffset": 96, "byteLength": 6 }
        ]
    })
}

fn albedo_png() -> RgbaImage {
    let mut image = RgbaImage::new(2, 2);
    for pixel in image.pixels_mut() {
        *pixel = Rgba([200u8, 100, 50, 255]);
    }
    image
}

fn metallic_roughness_png() -> RgbaImage {
    // Blue channel = metallic, green channel = roughness.
    let mut image = RgbaImage::new(1, 1);
    image.put_pixel(0, 0, Rgba([0u8, 128, 255, 255]));
    image
}

#[test]
fn imports_gltf_with_external_assets() {
    let root = temp_assets("external");
    let models_dir = root.join("Assets").join("Models");

    // External `.bin`, padded to 4 bytes per the glTF spec.
    let mut bin = mesh_bin();
    while bin.len() % 4 != 0 {
        bin.push(0);
    }
    std::fs::write(models_dir.join("triangle.bin"), &bin).expect("write bin");

    albedo_png()
        .save(models_dir.join("albedo.png"))
        .expect("write albedo");
    metallic_roughness_png()
        .save(models_dir.join("mr.png"))
        .expect("write mr");

    let mut json = base_json();
    json["images"] = json!([{ "uri": "albedo.png" }, { "uri": "mr.png" }]);
    json["buffers"] = json!([{ "uri": "triangle.bin", "byteLength": 102 }]);
    std::fs::write(
        models_dir.join("triangle.gltf"),
        serde_json::to_vec_pretty(&json).expect("serialize gltf"),
    )
    .expect("write gltf");

    let file_manager = make_file_manager(&root);
    let result = GltfImporter::import_with_file_manager(
        &file_manager,
        GltfImportTask {
            file: "Models/triangle.gltf".into(),
            import: GltfImport::WholeFile,
        },
    );

    let _ = std::fs::remove_dir_all(&root);

    assert!(
        result.errors.is_empty(),
        "import errors: {:?}",
        result.errors
    );
    assert_eq!(result.models.len(), 1, "expected exactly one model");
    assert_eq!(result.models[0].mesh.indices.len(), 3);
}

#[test]
fn imports_glb_with_embedded_bin() {
    let root = temp_assets("glb");
    let models_dir = root.join("Assets").join("Models");

    // Mesh data followed by the two PNGs, all referenced through buffer views.
    let mut bin = mesh_bin();
    let png1_offset = bin.len();
    let mut albedo = Vec::new();
    albedo_png()
        .write_to(&mut Cursor::new(&mut albedo), ImageFormat::Png)
        .expect("encode albedo");
    let png2_offset = png1_offset + albedo.len();
    let mut mr = Vec::new();
    metallic_roughness_png()
        .write_to(&mut Cursor::new(&mut mr), ImageFormat::Png)
        .expect("encode mr");
    let total = png2_offset + mr.len();
    bin.extend_from_slice(&albedo);
    bin.extend_from_slice(&mr);

    let mut json = base_json();
    json["images"] = json!([
        { "bufferView": 4, "mimeType": "image/png" },
        { "bufferView": 5, "mimeType": "image/png" }
    ]);
    json["bufferViews"]
        .as_array_mut()
        .expect("bufferViews array")
        .extend([
            json!({ "buffer": 0, "byteOffset": png1_offset, "byteLength": albedo.len() }),
            json!({ "buffer": 0, "byteOffset": png2_offset, "byteLength": mr.len() }),
        ]);
    json["buffers"] = json!([{ "byteLength": total }]);

    let glb = build_glb(
        &serde_json::to_vec(&json).expect("serialize glb json"),
        &bin,
    );
    std::fs::write(models_dir.join("triangle.glb"), &glb).expect("write glb");

    let file_manager = make_file_manager(&root);
    let result = GltfImporter::import_with_file_manager(
        &file_manager,
        GltfImportTask {
            file: "Models/triangle.glb".into(),
            import: GltfImport::WholeFile,
        },
    );

    let _ = std::fs::remove_dir_all(&root);

    assert!(
        result.errors.is_empty(),
        "import errors: {:?}",
        result.errors
    );
    assert_eq!(result.models.len(), 1, "expected exactly one model");
    assert_eq!(result.models[0].mesh.indices.len(), 3);
}

/// Packs a glTF JSON document and a binary chunk into the GLB container format.
fn build_glb(json: &[u8], bin: &[u8]) -> Vec<u8> {
    fn pad4(bytes: &mut Vec<u8>) {
        while bytes.len() % 4 != 0 {
            bytes.push(0);
        }
    }

    let mut json_padded = json.to_vec();
    pad4(&mut json_padded);
    let mut bin_padded = bin.to_vec();
    pad4(&mut bin_padded);

    let total = 12 + 8 + json_padded.len() + 8 + bin_padded.len();

    let mut out = Vec::with_capacity(total);
    out.extend_from_slice(b"glTF");
    out.extend_from_slice(&2u32.to_le_bytes());
    out.extend_from_slice(&(total as u32).to_le_bytes());
    out.extend_from_slice(&(json.len() as u32).to_le_bytes());
    out.extend_from_slice(&0x4E4F534Au32.to_le_bytes()); // "JSON"
    out.extend_from_slice(&json_padded);
    out.extend_from_slice(&(bin.len() as u32).to_le_bytes());
    out.extend_from_slice(&0x004E4942u32.to_le_bytes()); // "BIN\0"
    out.extend_from_slice(&bin_padded);
    out
}

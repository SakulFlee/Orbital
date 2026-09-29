//! End-to-end tests for the glTF importer's cross-platform asset loading.
//!
//! Exercises the custom in-memory import path with a temp
//! [`FileManager`](orbital_file_manager::FileManager): a `.gltf` referencing
//! external `.bin`/`.png` files (the Android-relevant case) and a self-contained
//! `.glb` (the case the repo's examples use).

use std::io::Cursor;
use std::sync::Arc;

use image::{ImageFormat, Rgba, RgbaImage};
use orbital_file_manager::{DesktopAssetSource, DirStorage, FileManager};
use orbital_importer_gltf::{GltfImport, GltfImportTask, GltfImporter};
use orbital_material_shader::MaterialShaderDescriptor;
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

/// Images that no material references must not be decoded, while the
/// referenced ones still are.
///
/// Skipping is observable: an unreferenced image whose bytes are *not* valid
/// image data would fail to decode. Pointing `images[1]` at a file containing
/// garbage therefore asserts the importer never touched it, whereas the
/// pre-optimization behavior (decode everything) reports an error. This also
/// pins the positional invariant, since the referenced albedo stays at image
/// index 0 while the skipped image sits at index 1.
#[test]
fn skips_images_that_no_material_references() {
    let root = temp_assets("unused_images");
    let models_dir = root.join("Assets").join("Models");

    let mut bin = mesh_bin();
    while bin.len() % 4 != 0 {
        bin.push(0);
    }
    std::fs::write(models_dir.join("triangle.bin"), &bin).expect("write bin");

    // Referenced by the material's baseColorTexture (image 0).
    albedo_png()
        .save(models_dir.join("albedo.png"))
        .expect("write albedo");

    // Referenced by metallicRoughnessTexture (image 1).
    metallic_roughness_png()
        .save(models_dir.join("mr.png"))
        .expect("write mr");

    // Image 2 exists in the document and is listed in `images`, but no
    // material references it. Its contents are deliberately not a decodable
    // image, so decoding it would fail the import.
    std::fs::write(models_dir.join("unused.png"), b"not a png").expect("write unused");

    let mut json = base_json();
    json["images"] = json!([
        { "uri": "albedo.png" },
        { "uri": "mr.png" },
        { "uri": "unused.png" }
    ]);
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
        "import errors (an unreferenced image was decoded?): {:?}",
        result.errors
    );
    assert_eq!(result.models.len(), 1, "expected exactly one model");
    assert_eq!(result.models[0].mesh.indices.len(), 3);
}

/// The same skip must hold for images embedded in a `.glb` binary chunk, where
/// a skipped image costs a buffer-view slice rather than a file read.
#[test]
fn skips_unreferenced_images_in_glb() {
    let root = temp_assets("unused_glb");
    let models_dir = root.join("Assets").join("Models");

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
    let unused_offset = png2_offset + mr.len();
    // Not a decodable image: decoding this slice would fail the import.
    let unused: Vec<u8> = b"not a png".to_vec();
    let total = unused_offset + unused.len();
    bin.extend_from_slice(&albedo);
    bin.extend_from_slice(&mr);
    bin.extend_from_slice(&unused);

    let mut json = base_json();
    json["images"] = json!([
        { "bufferView": 4, "mimeType": "image/png" },
        { "bufferView": 5, "mimeType": "image/png" },
        { "bufferView": 6, "mimeType": "image/png" }
    ]);
    json["bufferViews"]
        .as_array_mut()
        .expect("bufferViews array")
        .extend([
            json!({ "buffer": 0, "byteOffset": png1_offset, "byteLength": albedo.len() }),
            json!({ "buffer": 0, "byteOffset": png2_offset, "byteLength": mr.len() }),
            json!({ "buffer": 0, "byteOffset": unused_offset, "byteLength": unused.len() }),
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
        "import errors (an unreferenced image was decoded?): {:?}",
        result.errors
    );
    assert_eq!(result.models.len(), 1, "expected exactly one model");
    assert_eq!(result.models[0].mesh.indices.len(), 3);
}

/// Primitives that share a material must still each get a correct, complete
/// material — the per-mesh material cache must not hand one primitive's
/// material to another, nor let a later mutation leak between them.
///
/// Two primitives reference the same material, and a third references a
/// *different* material built from different textures. Importing must produce
/// three models whose materials match what each primitive's own material
/// declares: the two sharing primitives identically, and the third differently.
#[test]
fn shared_material_primitives_each_get_correct_material() {
    let root = temp_assets("shared_material");
    let models_dir = root.join("Assets").join("Models");

    let mut bin = mesh_bin();
    while bin.len() % 4 != 0 {
        bin.push(0);
    }
    std::fs::write(models_dir.join("triangle.bin"), &bin).expect("write bin");

    // Four distinct textures: material 0 uses (albedo_a, mr_a), material 1 uses
    // (albedo_b, mr_b).
    for (name, rgba) in [
        ("albedo_a.png", [200u8, 100, 50, 255]),
        ("mr_a.png", [0u8, 128, 255, 255]),
        ("albedo_b.png", [10u8, 220, 30, 255]),
        ("mr_b.png", [255u8, 0, 64, 255]),
    ] {
        let mut image = RgbaImage::new(2, 2);
        for pixel in image.pixels_mut() {
            *pixel = Rgba(rgba);
        }
        image.save(models_dir.join(name)).expect("write png");
    }

    let mut json = base_json();
    // Two primitives share material 0; the third uses material 1.
    json["meshes"] = json!([{ "primitives": [
        { "attributes": { "POSITION": 0, "NORMAL": 1, "TEXCOORD_0": 2 }, "indices": 3, "material": 0 },
        { "attributes": { "POSITION": 0, "NORMAL": 1, "TEXCOORD_0": 2 }, "indices": 3, "material": 0 },
        { "attributes": { "POSITION": 0, "NORMAL": 1, "TEXCOORD_0": 2 }, "indices": 3, "material": 1 }
    ] }]);
    json["materials"] = json!([
        { "name": "A", "pbrMetallicRoughness": {
            "baseColorTexture": { "index": 0 },
            "metallicRoughnessTexture": { "index": 1 }
        } },
        { "name": "B", "pbrMetallicRoughness": {
            "baseColorTexture": { "index": 2 },
            "metallicRoughnessTexture": { "index": 3 }
        } }
    ]);
    json["textures"] = json!([
        { "source": 0 }, { "source": 1 }, { "source": 2 }, { "source": 3 }
    ]);
    json["images"] = json!([
        { "uri": "albedo_a.png" }, { "uri": "mr_a.png" },
        { "uri": "albedo_b.png" }, { "uri": "mr_b.png" }
    ]);
    json["buffers"] = json!([{ "uri": "triangle.bin", "byteLength": 102 }]);
    std::fs::write(models_dir.join("triangle.bin"), &bin).expect("write bin");
    std::fs::write(
        models_dir.join("shared.gltf"),
        serde_json::to_vec_pretty(&json).expect("serialize gltf"),
    )
    .expect("write gltf");

    let file_manager = make_file_manager(&root);
    let result = GltfImporter::import_with_file_manager(
        &file_manager,
        GltfImportTask {
            file: "Models/shared.gltf".into(),
            import: GltfImport::WholeFile,
        },
    );

    let _ = std::fs::remove_dir_all(&root);

    assert!(
        result.errors.is_empty(),
        "import errors: {:?}",
        result.errors
    );
    assert_eq!(result.models.len(), 3, "expected one model per primitive");

    // Each model carries its material in a list; take the single entry.
    let material_of = |model: &orbital_model::ModelDescriptor| -> Arc<MaterialShaderDescriptor> {
        assert_eq!(
            model.materials.len(),
            1,
            "expected exactly one material per model"
        );
        Arc::clone(&model.materials[0])
    };
    let first = material_of(&result.models[0]);
    let second = material_of(&result.models[1]);
    let third = material_of(&result.models[2]);

    // The two primitives sharing material 0 must be identical...
    assert_eq!(
        first, second,
        "primitives sharing a material must receive the same material"
    );

    // ...and the third, which uses material 1 with different textures, must not
    // have been handed material 0's textures.
    assert_ne!(
        first, third,
        "a primitive with a different material must not receive a cached one"
    );

    // Sanity: the albedo texture's pixel data must be present, not silently
    // empty. Textures reach the material through its shader variables, the
    // first of which is the albedo (see `parse_materials`). Matched on the
    // Debug rendering so the test needs no extra crate dependency.
    for (label, material) in [("first", &first), ("second", &second), ("third", &third)] {
        let rendered = format!("{:?}", material.variables.first());
        assert!(
            rendered.contains("Data { pixels: ["),
            "{label}: expected a Data texture with pixel data, got {rendered}"
        );
        assert!(
            !rendered.contains("pixels: []"),
            "{label}: albedo pixels are empty: {rendered}"
        );
    }
}

/// Packs a glTF JSON document and a binary chunk into the GLB container format.
///
/// Both chunks are declared at their 4-byte-aligned length, matching what
/// `gltf`'s own `Glb::to_writer` emits. The reader splits the JSON at exactly
/// the declared length and hands it straight to `serde_json`, so the JSON
/// padding has to be insignificant whitespace: NUL padding would be rejected as
/// trailing characters, while spaces parse fine.
fn build_glb(json: &[u8], bin: &[u8]) -> Vec<u8> {
    fn pad4(bytes: &mut Vec<u8>, fill: u8) {
        while bytes.len() % 4 != 0 {
            bytes.push(fill);
        }
    }

    // Space is insignificant JSON whitespace; the BIN chunk is raw bytes, so
    // its padding value is irrelevant.
    let mut json_padded = json.to_vec();
    pad4(&mut json_padded, b' ');
    let mut bin_padded = bin.to_vec();
    pad4(&mut bin_padded, 0);

    let total = 12 + 8 + json_padded.len() + 8 + bin_padded.len();

    let mut out = Vec::with_capacity(total);
    out.extend_from_slice(b"glTF");
    out.extend_from_slice(&2u32.to_le_bytes());
    out.extend_from_slice(&(total as u32).to_le_bytes());
    out.extend_from_slice(&(json_padded.len() as u32).to_le_bytes());
    out.extend_from_slice(&0x4E4F534Au32.to_le_bytes()); // "JSON"
    out.extend_from_slice(&json_padded);
    out.extend_from_slice(&(bin_padded.len() as u32).to_le_bytes());
    out.extend_from_slice(&0x004E4942u32.to_le_bytes()); // "BIN\0"
    out.extend_from_slice(&bin_padded);
    out
}

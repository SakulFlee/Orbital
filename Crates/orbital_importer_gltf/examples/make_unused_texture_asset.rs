//! Builds a glTF asset with unreferenced textures, for measuring the importer.
//!
//! Writes `<out-dir>/Assets/Models/unused_textures.glb`: a textured triangle
//! plus N large images that no material points at. Timing an import of this
//! asset shows the cost the skip-unreferenced-images optimization removes.
//!
//! Run with:
//!   cargo run --release -p orbital_importer_gltf --example make_unused_texture_asset -- <out-dir> [count] [size]

use std::io::Cursor;

use image::{ImageFormat, Rgba, RgbaImage};
use serde_json::json;

fn pad4(bytes: &mut Vec<u8>, fill: u8) {
    while !bytes.len().is_multiple_of(4) {
        bytes.push(fill);
    }
}

fn build_glb(json: &[u8], bin: &[u8]) -> Vec<u8> {
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

/// A 3-vertex triangle: positions (3xvec3f32), normals (3xvec3f32),
/// uvs (3xvec2f32), indices (3xu16) = 102 bytes, tightly packed.
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

fn png(width: u32, height: u32, rgba: [u8; 4]) -> Vec<u8> {
    let mut out = Vec::new();
    RgbaImage::from_pixel(width, height, Rgba(rgba))
        .write_to(&mut Cursor::new(&mut out), ImageFormat::Png)
        .expect("encode png");
    out
}

fn main() {
    let mut args = std::env::args().skip(1);
    let out_dir = args
        .next()
        .expect("usage: make_unused_texture_asset <out-dir> [count] [size]");
    let unused_count: usize = args
        .next()
        .map(|x| x.parse().expect("count must be a number"))
        .unwrap_or(4);
    let size: u32 = args
        .next()
        .map(|x| x.parse().expect("size must be a number"))
        .unwrap_or(1024);

    let models_dir = std::path::Path::new(&out_dir).join("Assets").join("Models");
    std::fs::create_dir_all(&models_dir).expect("create Models dir");

    // Image 0 (albedo) and image 1 (metallic-roughness) are referenced by the
    // material and are full-size, so the measured import reflects a realistic
    // texture working set rather than two near-empty placeholder images. The
    // rest are deliberately unreferenced.
    let mut bin = mesh_bin();
    let albedo_offset = bin.len();
    let albedo = png(size, size, [200, 100, 50, 255]);
    let mr_offset = albedo_offset + albedo.len();
    let mr = png(size, size, [0, 128, 255, 255]);
    bin.extend_from_slice(&albedo);
    bin.extend_from_slice(&mr);

    let mut buffer_views = vec![
        json!({ "buffer": 0, "byteOffset": 0, "byteLength": 36 }),
        json!({ "buffer": 0, "byteOffset": 36, "byteLength": 36 }),
        json!({ "buffer": 0, "byteOffset": 72, "byteLength": 24 }),
        json!({ "buffer": 0, "byteOffset": 96, "byteLength": 6 }),
        json!({ "buffer": 0, "byteOffset": albedo_offset, "byteLength": albedo.len() }),
        json!({ "buffer": 0, "byteOffset": mr_offset, "byteLength": mr.len() }),
    ];
    let mut images = vec![
        json!({ "bufferView": 4, "mimeType": "image/png" }),
        json!({ "bufferView": 5, "mimeType": "image/png" }),
    ];
    let mut textures = vec![json!({ "source": 0 }), json!({ "source": 1 })];

    for i in 0..unused_count {
        let offset = bin.len();
        // Vary the colour so the PNGs do not compress to identical bytes.
        let seed = (i as u8).wrapping_mul(37).wrapping_add(11);
        let encoded = png(
            size,
            size,
            [seed, seed.wrapping_add(60), seed.wrapping_add(120), 255],
        );
        buffer_views.push(json!({
            "buffer": 0, "byteOffset": offset, "byteLength": encoded.len()
        }));
        images.push(json!({ "bufferView": buffer_views.len() - 1, "mimeType": "image/png" }));
        // A texture entry pointing at it, but no material referencing that
        // texture: this is the realistic shape of an asset with leftover art.
        textures.push(json!({ "source": images.len() - 1 }));
        bin.extend_from_slice(&encoded);
    }

    let document = json!({
        "asset": { "version": "2.0", "generator": "orbital_unused_texture_asset" },
        "scene": 0,
        "scenes": [ { "nodes": [0] } ],
        "nodes": [ { "mesh": 0, "name": "Triangle" } ],
        "meshes": [ { "primitives": [ {
            "attributes": { "POSITION": 0, "NORMAL": 1, "TEXCOORD_0": 2 },
            "indices": 3,
            "material": 0
        } ] } ],
        "materials": [ { "name": "Base", "pbrMetallicRoughness": {
            "baseColorTexture": { "index": 0 },
            "metallicRoughnessTexture": { "index": 1 }
        } } ],
        "textures": textures,
        "images": images,
        "accessors": [
            { "bufferView": 0, "componentType": 5126, "count": 3, "type": "VEC3",
              "min": [0.0, 0.0, 0.0], "max": [1.0, 1.0, 0.0] },
            { "bufferView": 1, "componentType": 5126, "count": 3, "type": "VEC3" },
            { "bufferView": 2, "componentType": 5126, "count": 3, "type": "VEC2" },
            { "bufferView": 3, "componentType": 5123, "count": 3, "type": "SCALAR" }
        ],
        "bufferViews": buffer_views,
        "buffers": [ { "byteLength": bin.len() } ]
    });

    let glb = build_glb(&serde_json::to_vec(&document).expect("serialize"), &bin);
    let path = models_dir.join("unused_textures.glb");
    std::fs::write(&path, &glb).expect("write glb");

    println!("wrote {}", path.display());
    println!(
        "  images: {} ({} referenced by the material, {unused_count} unreferenced)",
        images.len(),
        2
    );
    println!("  unreferenced texture size: {size}x{size}");
    println!(
        "  file size: {:.2} MB",
        glb.len() as f64 / (1024.0 * 1024.0)
    );
}

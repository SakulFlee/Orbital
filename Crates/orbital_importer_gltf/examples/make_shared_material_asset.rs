//! Builds a glTF asset with many primitives sharing one material, for
//! measuring the importer.
//!
//! Writes `<out-dir>/Assets/Models/shared_material.glb`: N primitives that all
//! reference the *same* material, and therefore the same textures. Timing an
//! import of this asset shows the cost of rebuilding identical texture
//! descriptors once per primitive.
//!
//! Run with:
//!   cargo run --release -p orbital_importer_gltf --example make_shared_material_asset -- <out-dir> [primitives] [tex_size]

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

fn png(size: u32, rgba: [u8; 4]) -> Vec<u8> {
    let mut out = Vec::new();
    RgbaImage::from_pixel(size, size, Rgba(rgba))
        .write_to(&mut Cursor::new(&mut out), ImageFormat::Png)
        .expect("encode png");
    out
}

/// A 3-vertex triangle: positions, normals, uvs, indices = 102 bytes.
fn triangle_bin() -> Vec<u8> {
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

fn main() {
    let mut args = std::env::args().skip(1);
    let out_dir = args
        .next()
        .expect("usage: make_shared_material_asset <out-dir> [primitives] [tex_size]");
    let primitive_count: usize = args
        .next()
        .map(|x| x.parse().expect("primitives must be a number"))
        .unwrap_or(32);
    let tex_size: u32 = args
        .next()
        .map(|x| x.parse().expect("tex_size must be a number"))
        .unwrap_or(1024);

    let models_dir = std::path::Path::new(&out_dir).join("Assets").join("Models");
    std::fs::create_dir_all(&models_dir).expect("create Models dir");

    let mut bin: Vec<u8> = Vec::new();
    let mut buffer_views: Vec<serde_json::Value> = Vec::new();
    let mut accessors: Vec<serde_json::Value> = Vec::new();
    let mut primitives: Vec<serde_json::Value> = Vec::new();

    // One copy of the triangle data, referenced by every primitive. All four
    // accessors share a single buffer view, so the two texture buffer views
    // land at indices 1 and 2.
    let shared_offset = bin.len();
    bin.extend_from_slice(&triangle_bin());
    buffer_views.push(json!({ "buffer": 0, "byteOffset": shared_offset, "byteLength": 102 }));
    let mesh_view = buffer_views.len() - 1;
    accessors.push(json!({
        "bufferView": mesh_view, "componentType": 5126, "count": 3, "type": "VEC3",
        "min": [0.0, 0.0, 0.0], "max": [1.0, 1.0, 0.0]
    }));
    accessors.push(json!({
        "bufferView": mesh_view, "componentType": 5126, "count": 3, "type": "VEC3"
    }));
    accessors.push(json!({
        "bufferView": mesh_view, "componentType": 5126, "count": 3, "type": "VEC2"
    }));
    accessors.push(json!({
        "bufferView": mesh_view, "componentType": 5123, "count": 3, "type": "SCALAR"
    }));

    for _ in 0..primitive_count {
        primitives.push(json!({
            "attributes": { "POSITION": 0, "NORMAL": 1, "TEXCOORD_0": 2 },
            "indices": 3,
            "material": 0
        }));
    }

    // Two textures, both referenced by the one shared material.
    let albedo_offset = bin.len();
    let albedo = png(tex_size, [200, 100, 50, 255]);
    bin.extend_from_slice(&albedo);
    buffer_views.push(json!({
        "buffer": 0, "byteOffset": albedo_offset, "byteLength": albedo.len()
    }));
    let mr_offset = bin.len();
    let mr = png(tex_size, [0, 128, 255, 255]);
    bin.extend_from_slice(&mr);
    buffer_views.push(json!({
        "buffer": 0, "byteOffset": mr_offset, "byteLength": mr.len()
    }));

    let document = json!({
        "asset": { "version": "2.0", "generator": "orbital_shared_material_asset" },
        "scene": 0,
        "scenes": [ { "nodes": [0] } ],
        "nodes": [ { "mesh": 0, "name": "SharedMaterialMesh" } ],
        "meshes": [ { "primitives": primitives } ],
        "materials": [ { "name": "Shared", "pbrMetallicRoughness": {
            "baseColorTexture": { "index": 0 },
            "metallicRoughnessTexture": { "index": 1 }
        } } ],
        "textures": [ { "source": 0 }, { "source": 1 } ],
        "images": [
            { "bufferView": 1, "mimeType": "image/png" },
            { "bufferView": 2, "mimeType": "image/png" }
        ],
        "accessors": accessors,
        "bufferViews": buffer_views,
        "buffers": [ { "byteLength": bin.len() } ]
    });

    let glb = build_glb(&serde_json::to_vec(&document).expect("serialize"), &bin);
    let path = models_dir.join("shared_material.glb");
    std::fs::write(&path, &glb).expect("write glb");

    println!("wrote {}", path.display());
    println!("  primitives sharing one material: {primitive_count}");
    println!("  shared textures: 2 x {tex_size}x{tex_size}");
    println!(
        "  file size: {:.2} MB",
        glb.len() as f64 / (1024.0 * 1024.0)
    );
}

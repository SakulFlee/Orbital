//! End-to-end benchmark for the glTF importer.
//!
//! Run with:
//!   cargo run --release --example import_bench -p orbital_importer_gltf -- <file.glb> [rounds]
//!
//! Times [`GltfImporter::import`] — document parse, buffer load, image decode,
//! geometry and material parsing — which is the cost that matters in
//! production.
//!
//! ## Reading the numbers
//!
//! Import time is sensitive to machine load. On a busy machine the same import
//! can vary by well over 2x, which is wider than the differences this benchmark
//! exists to resolve. Contention can only *add* time, never remove it, so the
//! **minimum** across rounds is the most reliable estimator of uncontended
//! cost, and that is the headline figure. Medians and per-round samples are
//! still printed, so a noisy environment is visible rather than silently
//! skewing the result.

use std::time::Instant;

use orbital_file_manager::{DesktopAssetSource, DirStorage, FileManager};
use orbital_importer_gltf::{GltfImport, GltfImportTask, GltfImporter};

/// Indices of the images actually referenced by a material.
///
/// Reported for context: images that no material references are the work the
/// skip-unreferenced-images optimization avoids, so `never referenced > 0`
/// indicates potential savings rather than a problem with the asset.
fn referenced_images(document: &gltf::Document) -> std::collections::BTreeSet<usize> {
    let mut referenced = std::collections::BTreeSet::new();
    for material in document.materials() {
        if let Some(info) = material.normal_texture() {
            referenced.insert(info.texture().source().index());
        }
        if let Some(info) = material.pbr_metallic_roughness().base_color_texture() {
            referenced.insert(info.texture().source().index());
        }
        if let Some(info) = material
            .pbr_metallic_roughness()
            .metallic_roughness_texture()
        {
            referenced.insert(info.texture().source().index());
        }
        if let Some(info) = material.occlusion_texture() {
            referenced.insert(info.texture().source().index());
        }
        if let Some(info) = material.emissive_texture() {
            referenced.insert(info.texture().source().index());
        }
    }
    referenced
}

fn median(values: &mut [f64]) -> f64 {
    values.sort_by(|a, b| a.partial_cmp(b).expect("timings are comparable"));
    values[values.len() / 2]
}

fn main() {
    let mut args = std::env::args().skip(1);
    let path = args
        .next()
        .expect("usage: import_bench <file.glb> [rounds]");
    let rounds: usize = args
        .next()
        .map(|x| x.parse().expect("rounds must be a number"))
        .unwrap_or(15);

    let bytes = std::fs::read(&path).expect("failed to read glb");
    let gltf::Gltf { document, .. } = gltf::Gltf::from_slice(&bytes).expect("failed to parse glb");
    let image_count = document.images().count();
    let referenced = referenced_images(&document);
    let material_count = document.materials().count();
    let primitive_count = document.meshes().flat_map(|m| m.primitives()).count();

    println!("file            : {path}");
    println!(
        "size            : {:.2} MB",
        bytes.len() as f64 / (1024.0 * 1024.0)
    );
    println!("images          : {image_count}");
    println!("referenced      : {referenced:?}");
    println!("never referenced: {}", image_count - referenced.len());
    println!("materials       : {material_count}");
    println!("primitives      : {primitive_count}");
    println!("rounds          : {rounds}");
    println!();

    // The importer resolves assets through a FileManager whose asset root is
    // `<root>/Assets`, and asset paths are relative to that root, so a model at
    // `Assets/Models/foo.glb` is addressed as `Models/foo.glb`.
    let model_path = std::path::Path::new(&path);
    let model_dir = model_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_default();
    let (root, asset_path) = if model_dir.ends_with("Assets/Models") {
        (
            model_dir
                .parent()
                .and_then(|assets_dir| assets_dir.parent())
                .map(|p| p.to_path_buf())
                .unwrap_or_default(),
            format!(
                "Models/{}",
                model_path
                    .file_name()
                    .expect("model has a file name")
                    .to_string_lossy()
            ),
        )
    } else {
        (std::path::PathBuf::from("."), path.clone())
    };

    let asset_root = root.join("Assets");
    let file_manager = FileManager::new(
        Box::new(DesktopAssetSource::with_base_dir(asset_root.clone())),
        Box::new(DirStorage::new(std::env::temp_dir())),
    );

    // Fail fast if the asset cannot be read: a benchmark that silently measures
    // error paths would report ~0 ms and look like an enormous speedup.
    if let Err(error) = file_manager.read_asset_bytes(&asset_path) {
        eprintln!(
            "fatal: cannot read asset '{asset_path}' under root '{}': {error}",
            asset_root.display()
        );
        std::process::exit(1);
    }

    let import = || {
        GltfImporter::import_with_file_manager(
            &file_manager,
            GltfImportTask {
                file: asset_path.clone(),
                import: GltfImport::WholeFile,
            },
        )
    };

    // Warm up so first-call page faults and lazy initialization inside
    // wgpu/image are not attributed to the measured runs.
    let warmup = import();
    if !warmup.errors.is_empty() {
        eprintln!("fatal: warmup import reported errors:");
        for error in &warmup.errors {
            eprintln!("  {error}");
        }
        std::process::exit(1);
    }

    let mut samples = Vec::with_capacity(rounds);
    let mut models = 0usize;
    for _ in 0..rounds {
        let start = Instant::now();
        let result = import();
        samples.push(start.elapsed().as_secs_f64() * 1000.0);
        models = result.models.len();
        if !result.errors.is_empty() {
            eprintln!("fatal: import reported errors:");
            for error in &result.errors {
                eprintln!("  {error}");
            }
            std::process::exit(1);
        }
    }

    if models == 0 {
        eprintln!("fatal: import produced no models; nothing was measured");
        std::process::exit(1);
    }

    let mut sorted = samples.clone();
    let median_ms = median(&mut sorted);
    let min_ms = sorted[0];
    let max_ms = sorted[sorted.len() - 1];

    println!("models imported: {models}");
    println!(
        "import time: min {min_ms:.2} ms | median {median_ms:.2} ms | max {max_ms:.2} ms  (n={rounds})"
    );
    println!();
    println!("per-run samples (ms):");
    for (i, sample) in samples.iter().enumerate() {
        println!("  [{i:>2}] {sample:>8.2}");
    }
}

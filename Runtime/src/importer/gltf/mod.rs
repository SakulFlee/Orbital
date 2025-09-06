use crate::resources::{
    CameraDescriptor, FilterMode, MaterialDescriptor, MeshDescriptor, ModelDescriptor,
    PBRMaterialDescriptor, TextureDescriptor, TextureSize, Transform, Vertex,
};
use cgmath::{InnerSpace, Point3, Quaternion, Vector2, Vector3, Zero};
use gltf::camera::Projection;
use gltf::image::Format;
use gltf::{Camera, Document, Material, Mesh, Node, Scene, Semantic};
use log::warn;
use std::error::Error;
use std::sync::Arc;
use wgpu::TextureFormat::R32Float;
use wgpu::{Color, TextureDimension, TextureFormat, TextureUsages, TextureViewDimension};

mod import;
pub use import::*;

mod specific_import;
pub use specific_import::*;

mod task;
pub use task::*;

mod import_type;
pub use import_type::*;

mod result;
pub use result::*;

mod error;
use crate::quaternion::quaternion_to_pitch_yaw;
pub use error::*;

// Import the new tangent utility function
mod tangent_utils;

#[cfg(test)]
mod tests;

/// Used to load/import "things" from a glTF file.
/// This should support most variants of glTF files but not necessarily everything.
///
/// Since most imports require labels to be used, make sure that your glTF file does include support
/// for labels! Labels is an _optional feature_ in glTF files. Most applications export glTF files
/// directly with the label without any modification being necessary, other apps might have a toggle.
///
/// # Known unsupported behaviors:
/// - URL references to websites, e.g. to download an image, are not supported.
///   Any resources are required to be local and accessible.
#[derive(Debug)]
pub struct GltfImporter;

impl GltfImporter {
    /// Starts the import of a glTF file given settings defined in [`GltfImportTask`].
    ///
    /// This function will return a [`GltfImportResult`] which is different from the classical
    /// [`Result`] type Rust commonly uses.
    /// The reason for this is that a glTF file can contain many different types of targets to import
    /// such as Cameras, Meshes/Models, Materials, Lights, etc.
    /// The normally used [`Result`] type would result in an import ending as soon as the first issue
    /// is found. Using the [`GltfImportResult`] struct allows us to still try importing everything
    /// else before failing. The user then can decide if the result is acceptable or not.
    ///
    /// Note that this does **NOT** handle instancing automatically.
    /// If a given model/mesh (_glTF primitive mesh_) is in the scene multiple times, it will also be
    /// imported multiple times, based on the [`GltfImportTask`].
    /// If you are aware of a model being used multiple times in a scene, import it once, then modify
    /// the transform to match your instances.
    /// _This feature will be implemented in the future: [#292](https://github.com/SakulFlee/Orbital/issues/292)!_
    pub async fn import(import_task: GltfImportTask) -> GltfImportResult {
        let (document, buffers, textures) = match gltf::import(&import_task.file) {
            Ok(x) => x,
            Err(e) => {
                return GltfImportResult {
                    errors: vec![Box::new(e)],
                    ..Default::default()
                }
            }
        };

        match import_task.import {
            GltfImport::WholeFile => Self::import_whole_file(&document, &buffers, &textures),
            GltfImport::Specific(specific_gltf_imports) => {
                let mut result = GltfImportResult::empty();

                for specific_import in specific_gltf_imports {
                    let import_result =
                        Self::import_specific(specific_import, &document, &buffers, &textures);
                    result.extend(import_result);
                }

                result
            }
        }
    }

    /// Handles importing from a glTF [`Document`] given a [`SpecificGltfImport`].
    fn import_specific(
        specific_import: SpecificGltfImport,
        document: &Document,
        buffers: &Vec<gltf::buffer::Data>,
        textures: &Vec<gltf::image::Data>,
    ) -> GltfImportResult {
        let mut result = GltfImportResult::empty();

        match specific_import.import_type {
            GltfImportType::Scene => {
                if let Some(scene) = document
                    .scenes()
                    .find(|scene| scene.name().is_some_and(|x| x == specific_import.label))
                {
                    let import_result =
                        Self::import_whole_scene(scene, document, buffers, textures);
                    result.extend(import_result);
                } else {
                    result
                        .errors
                        .push(Box::new(GltfError::NotFound(specific_import)));
                }
            }
            GltfImportType::Model | GltfImportType::Camera => {
                if let Some(node) = document.scenes().find_map(|scene| {
                    scene.nodes().find(|node| {
                        node.name()
                            .is_some_and(|name| name == specific_import.label)
                    })
                }) {
                    let import_result = Self::import_nodes(vec![node], buffers, textures);
                    result.extend(import_result);
                } else {
                    result
                        .errors
                        .push(Box::new(GltfError::NotFound(specific_import)));
                }
            }
        }

        result
    }

    /// Handles importing a whole glTF file
    fn import_whole_file(
        document: &Document,
        buffers: &Vec<gltf::buffer::Data>,
        textures: &Vec<gltf::image::Data>,
    ) -> GltfImportResult {
        let mut result = GltfImportResult::empty();

        for scene in document.scenes() {
            let import_result = Self::import_whole_scene(scene, document, buffers, textures);
            result.extend(import_result);
        }

        result
    }

    /// Handles importing a whole scene from a glTF [`Document`].
    fn import_whole_scene(
        scene: Scene,
        document: &Document,
        buffers: &Vec<gltf::buffer::Data>,
        textures: &Vec<gltf::image::Data>,
    ) -> GltfImportResult {
        let nodes: Vec<_> = scene.nodes().collect();

        Self::import_nodes(nodes, buffers, textures)
    }

    /// Handles importing a specific set of [`Node`]s from a glTF [`Document`].
    fn import_nodes(
        nodes: Vec<Node>,
        buffers: &Vec<gltf::buffer::Data>,
        textures: &Vec<gltf::image::Data>,
    ) -> GltfImportResult {
        let mut model_descriptors = Vec::new();
        let mut camera_descriptors = Vec::new();
        let mut errors = Vec::new();

        for node in nodes {
            if let Some(mesh) = node.mesh() {
                match Self::parse_models(&node, &mesh, buffers, textures) {
                    Ok(models) => model_descriptors.extend(models),
                    Err(e) => errors.push(e),
                }
            } else if let Some(camera) = node.camera() {
                match Self::parse_camera(&node, &camera, buffers) {
                    Ok(camera) => camera_descriptors.push(camera),
                    Err(e) => errors.push(e),
                }
            } else {
                warn!("Unknown node type: {node:?}");
            }
        }

        GltfImportResult {
            models: model_descriptors,
            cameras: camera_descriptors,
            errors,
        }
    }

    /// Returns the Orbital/WGPU [`TextureFormat`] as well as a boolean.
    /// If the boolean is true, that means an alpha channel has to be generated.
    /// This is due to WGPU not supporting RGB textures without an alpha channel.
    fn gltf_texture_format_to_orbital(format: Format) -> (TextureFormat, bool) {
        match format {
            Format::R8 => (TextureFormat::R8Unorm, false),
            Format::R8G8 => (TextureFormat::Rg8Unorm, false),
            Format::R8G8B8 => (TextureFormat::Rgba8Unorm, true),
            Format::R8G8B8A8 => (TextureFormat::Rgba8Unorm, false),
            Format::R16 => (TextureFormat::R16Unorm, false),
            Format::R16G16 => (TextureFormat::Rg16Unorm, false),
            Format::R16G16B16 => (TextureFormat::Rgba16Unorm, true),
            Format::R16G16B16A16 => (TextureFormat::Rgba16Unorm, false),
            Format::R32G32B32FLOAT => (TextureFormat::Rgba32Float, true),
            Format::R32G32B32A32FLOAT => (TextureFormat::Rgba32Float, false),
        }
    }

    /// Handles parsing of glTF textures ([`gltf::image::Data`]) and turns it into a [`TextureDescriptor`].
    fn parse_texture(data: &gltf::image::Data) -> TextureDescriptor {
        let (format, need_alpha_channel) = Self::gltf_texture_format_to_orbital(data.format);

        // Debug log to verify the determined format
        log::debug!(
            "Parsing texture: glTF format {:?} -> Orbital format {:?}, need_alpha: {}",
            data.format,
            format,
            need_alpha_channel
        );

        // The `pixels` data to be uploaded. Initialize with original data.
        let (pixel_data, width, height) = if need_alpha_channel {
            // If an alpha channel needs to be generated (e.g., RGB -> RGBA),
            // process the original data.
            // The gltf crate provides data without alpha in the cases where
            // `need_alpha_channel` is true.
            // wgpu requires a 4-component format (e.g., Rgba8Unorm), so we need to add alpha.
            // Standard assumption: Add an opaque alpha channel (255).
            let original_width = data.width;
            let original_height = data.height;

            // Calculate the number of channels in the source format
            let source_channels = match data.format {
                Format::R8 => 1,
                Format::R8G8 => 2,
                Format::R8G8B8 => 3,
                // These shouldn't happen when need_alpha_channel is true, but just in case:
                Format::R8G8B8A8 => 4,
                Format::R16 => 1,
                Format::R16G16 => 2,
                Format::R16G16B16 => 3,
                // These shouldn't happen when need_alpha_channel is true, but just in case:
                Format::R16G16B16A16 => 4,
                Format::R32G32B32FLOAT => 3,
                // These shouldn't happen when need_alpha_channel is true, but just in case:
                Format::R32G32B32A32FLOAT => 4,
            };

            // Calculate expected pixel count
            let pixel_count = (data.pixels.len() / source_channels) as u32;
            let expected_pixel_count = original_width * original_height;

            // Validate that we have the correct amount of data
            if pixel_count != expected_pixel_count {
                log::warn!(
                    "Texture data size mismatch: expected {} pixels ({}x{}), got {} pixels from {} bytes with {} channels",
                    expected_pixel_count,
                    original_width,
                    original_height,
                    pixel_count,
                    data.pixels.len(),
                    source_channels
                );
            }

            let mut processed_pixels = Vec::with_capacity(data.pixels.len() / source_channels * 4);
            // Iterate through chunks based on the source channel count
            for chunk in data.pixels.chunks(source_channels) {
                // Add all the source channels (R, G, B, etc.)
                for &byte in chunk {
                    processed_pixels.push(byte);
                }
                // Pad with zeros if needed to reach 3 channels (RGB)
                for _ in chunk.len()..3 {
                    processed_pixels.push(0u8);
                }
                // Add full alpha (255)
                processed_pixels.push(255u8);
            }

            // Debug log to verify the processed data size
            let expected_processed_size =
                (original_width as usize) * (original_height as usize) * 4;
            if processed_pixels.len() != expected_processed_size {
                log::warn!(
                    "Processed texture data size mismatch: expected {} bytes ({}x{}x4), got {} bytes",
                    expected_processed_size,
                    original_width,
                    original_height,
                    processed_pixels.len()
                );
            }

            (processed_pixels, original_width, original_height)
        } else {
            // No processing needed, use the original data.
            (data.pixels.clone(), data.width, data.height)
        };

        TextureDescriptor::Data {
            // Use the correctly determined pixel data
            pixels: pixel_data,
            size: TextureSize {
                width,
                height,
                depth_or_array_layers: 1, // A standard 2D texture has 1 layer
                base_mip: 0,
                mip_levels: 1, // glTF image data is typically just the base mip level
            },
            usages: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT, // Ensure required usages for textures
            format,
            // Determine dimension based on data. For glTF images, D2 is standard.
            texture_dimension: TextureDimension::D2,
            texture_view_dimension: TextureViewDimension::D2,
            filter_mode: FilterMode::linear(),
        }
    }

    /// Handles parsing a "dual" texture.
    /// Same as [`Self::parse_texture`], but splits the B(lue) and G(reen) channel into two separate
    /// textures according to the glTF specification for metallic-roughness textures.
    /// Metallic is in the B channel, Roughness is in the G channel.
    fn parse_dual_texture(data: &gltf::image::Data) -> (TextureDescriptor, TextureDescriptor) {
        let (format, need_alpha_channel) = Self::gltf_texture_format_to_orbital(data.format);

        // Calculate the number of channels in the source format
        let source_channels = match data.format {
            Format::R8 => 1,
            Format::R8G8 => 2,
            Format::R8G8B8 => 3,
            Format::R8G8B8A8 => 4,
            Format::R16 => 1,
            Format::R16G16 => 2,
            Format::R16G16B16 => 3,
            Format::R16G16B16A16 => 4,
            Format::R32G32B32FLOAT => 3,
            Format::R32G32B32A32FLOAT => 4,
        };

        let mut pixels_0 = Vec::with_capacity(data.pixels.len() / source_channels);
        let mut pixels_1 = Vec::with_capacity(data.pixels.len() / source_channels);

        for chunk in data.pixels.chunks(source_channels) {
            // According to glTF spec:
            // Blue channel (index 2) -> Metallic (texture_0)
            // Green channel (index 1) -> Roughness (texture_1)
            // First channel (Blue) -> Metallic
            if chunk.len() > 2 {
                pixels_0.push(chunk[2]); // Blue channel for metallic
            } else if chunk.len() > 0 {
                // If we don't have enough channels, use the first one
                pixels_0.push(chunk[0]);
            }
            // Second channel (Green) -> Roughness
            if chunk.len() > 1 {
                pixels_1.push(chunk[1]); // Green channel for roughness
            } else if chunk.len() > 0 {
                // If we don't have enough channels, use the first one
                pixels_1.push(chunk[0]);
            }
        }

        let actual_format = match data.format {
            Format::R8 | Format::R8G8 | Format::R8G8B8 | Format::R8G8B8A8 => TextureFormat::R8Unorm,
            Format::R16 | Format::R16G16 | Format::R16G16B16 | Format::R16G16B16A16 => {
                TextureFormat::R16Unorm
            }
            Format::R32G32B32FLOAT | Format::R32G32B32A32FLOAT => R32Float,
        };

        let texture_0 = TextureDescriptor::Data {
            pixels: pixels_0,
            size: TextureSize {
                width: data.width,
                height: data.height,
                depth_or_array_layers: 1,
                base_mip: 0,
                mip_levels: 1,
            },
            usages: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            format: actual_format,
            texture_dimension: TextureDimension::D2,
            texture_view_dimension: TextureViewDimension::D2,
            filter_mode: FilterMode::linear(),
        };
        let texture_1 = TextureDescriptor::Data {
            pixels: pixels_1,
            size: TextureSize {
                width: data.width,
                height: data.height,
                depth_or_array_layers: 1,
                base_mip: 0,
                mip_levels: 1,
            },
            usages: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            format: actual_format,
            texture_dimension: TextureDimension::D2,
            texture_view_dimension: TextureViewDimension::D2,
            filter_mode: FilterMode::linear(),
        };

        (texture_0, texture_1)
    }

    /// Handles parsing a glTF [`Material`] into an Orbital [`MaterialDescriptor`].
    fn parse_materials(
        material: &Material,
        textures: &Vec<gltf::image::Data>,
    ) -> MaterialDescriptor {
        let normal = if let Some(normal_info) = material.normal_texture() {
            Self::parse_texture(&textures[normal_info.texture().source().index()])
        } else {
            // Default normal map value: (0.5, 0.5, 1.0, 1.0) maps to (0, 0, 1) in tangent space after 2*x-1
            // Use linear format for normal maps (no sRGB conversion)
            let mut normal_desc = TextureDescriptor::uniform_rgba_value(0.5, 0.5, 1.0, 1.0, false);
            // Override the format to be linear instead of sRGB
            if let TextureDescriptor::Data { format, .. } = &mut normal_desc {
                *format = TextureFormat::Rgba8Unorm; // Linear format for normal maps
            }
            normal_desc
        };

        // NOTE: 'W' (Opacity / Transparency) is skipped here!
        let (albedo, albedo_factor) = if let Some(albedo_info) =
            material.pbr_metallic_roughness().base_color_texture()
        {
            let texture = Self::parse_texture(&textures[albedo_info.texture().source().index()]);
            let factor = material.pbr_metallic_roughness().base_color_factor();
            (texture, Vector3::new(factor[0], factor[1], factor[2]))
        } else {
            let factor = material.pbr_metallic_roughness().base_color_factor();
            let texture = TextureDescriptor::uniform_rgba_value(
                factor[0] as f64,
                factor[1] as f64,
                factor[2] as f64,
                factor[3] as f64,
                true,
            );

            (texture, Vector3::new(1.0, 1.0, 1.0))
        };

        let (metallic, roughness, metallic_factor, roughness_factor) =
            if let Some(metallic_and_roughness_info) = material
                .pbr_metallic_roughness()
                .metallic_roughness_texture()
            {
                // If a metallic & roughness texture is set, the factors will be needed to multiplied with the texture.

                let (texture_descriptor_metallic, texture_descriptor_roughness) =
                    Self::parse_dual_texture(
                        &textures[metallic_and_roughness_info.texture().source().index()],
                    );

                let factor_metallic = material.pbr_metallic_roughness().metallic_factor();
                let factor_roughness = material.pbr_metallic_roughness().roughness_factor();

                (
                    texture_descriptor_metallic,
                    texture_descriptor_roughness,
                    factor_metallic,
                    factor_roughness,
                )
            } else {
                // If no metallic/roughness texture is set, the factors will act as a global texture and the factors inside the shader should be set to 1.0!

                let factor_metallic = material.pbr_metallic_roughness().metallic_factor();
                let texture_descriptor_metallic = TextureDescriptor::uniform_rgba_value(
                    factor_metallic as f64,
                    0.0,
                    0.0,
                    1.0,
                    true,
                );

                let factor_roughness = material.pbr_metallic_roughness().roughness_factor();
                let texture_descriptor_roughness = TextureDescriptor::uniform_rgba_value(
                    factor_roughness as f64,
                    0.0,
                    0.0,
                    1.0,
                    true,
                );

                (
                    texture_descriptor_metallic,
                    texture_descriptor_roughness,
                    1.0,
                    1.0,
                )
            };

        let occlusion = if let Some(occlusion_info) = material.occlusion_texture() {
            Self::parse_texture(&textures[occlusion_info.texture().source().index()])
        } else {
            TextureDescriptor::uniform_rgba_white(false)
        };
        let emissive = if let Some(emissive_info) = material.emissive_texture() {
            Self::parse_texture(&textures[emissive_info.texture().source().index()])
        } else {
            let emissive_color = material.emissive_factor();
            TextureDescriptor::uniform_rgba_color(
                Color {
                    r: emissive_color[0] as f64,
                    g: emissive_color[1] as f64,
                    b: emissive_color[2] as f64,
                    a: 1.0,
                },
                true,
            )
        };

        let pbr_material = PBRMaterialDescriptor {
            name: material.name().map(|x| x.to_string()),
            normal,
            albedo,
            albedo_factor,
            metallic,
            metallic_factor,
            roughness,
            roughness_factor,
            occlusion,
            emissive,
            custom_material_shader: None,
        };

        pbr_material.into()
    }

    /// Handles parsing of a glTF [`Mesh`] into multiple [`ModelDescriptor`]s.
    /// A _glTF Primitive_ is what Orbital considers a [`Model`].
    /// A _glTF Attribute_ is, in some sense, what Orbital considers a [`Mesh`] and [`Vertex`]
    fn parse_models(
        node: &Node,
        mesh: &Mesh,
        buffers: &Vec<gltf::buffer::Data>,
        textures: &Vec<gltf::image::Data>,
    ) -> Result<Vec<ModelDescriptor>, Box<dyn Error>> {
        let primitives = mesh.primitives();
        let mut results = Vec::new();

        // glTF Primitive == Orbital Model
        for primitive in primitives {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            let Some(positions) = reader.read_positions() else {
                warn!("Primitive has no positions. Skipping mesh primitive.");
                continue;
            };
            let Some(indices) = reader.read_indices().map(|x| x.into_u32()) else {
                warn!("Primitive has no indices. Skipping mesh primitive.");
                continue;
            };
            let mut normals = reader.read_normals();
            let mut tangents = reader.read_tangents();
            let mut uvs = reader.read_tex_coords(0).map(|x| x.into_f32());
            primitive.attributes().for_each(|x| {
                if let Semantic::TexCoords(indices) = x.0 {
                    if indices > 1 {
                        warn!("More than one UV index found, only the first will be imported!");
                    }
                }
            });

            // TODO: CLEANUP

            // Collect all data into vectors first to avoid iterator issues
            let positions_vec: Vec<_> = positions.map(|p| Vector3::new(p[0], p[1], p[2])).collect();
            // Collect indices early as they are needed for normal calculation if normals are missing
            let indices_vec: Vec<u32> = reader
                .read_indices()
                .map(|x| x.into_u32())
                .map(|indices| indices.collect())
                .unwrap_or_default(); // Get indices_vec here

            // --- Normal Calculation Logic Start ---
            let normals_vec = if let Some(normals_iter) = normals {
                // If normals are provided in the glTF file, collect and convert them as before
                normals_iter
                    .map(|n| Vector3::new(n[0], n[1], n[2]))
                    .collect()
            } else {
                // If normals are *not* provided, calculate them generically
                warn!("Primitive has no normals. Calculating them based on triangle geometry.");

                // Ensure we have positions and indices to work with for normal calculation
                if positions_vec.is_empty() || indices_vec.is_empty() {
                    warn!("Cannot calculate normals: Missing positions or indices.");
                    (0..positions_vec.len()).map(|_| Vector3::zero()).collect()
                } else {
                    // Allocate a vector to accumulate normals for each vertex
                    let mut calculated_normals: Vec<Vector3<f32>> =
                        vec![Vector3::new(0.0, 0.0, 0.0); positions_vec.len()];

                    // Iterate through triangles using indices to calculate face normals
                    // Use indices_vec for calculating normals based on triangle connectivity.
                    for i in (0..indices_vec.len()).step_by(3) {
                        if i + 2 < indices_vec.len() {
                            let idx_a = indices_vec[i] as usize;
                            let idx_b = indices_vec[i + 1] as usize;
                            let idx_c = indices_vec[i + 2] as usize;

                            // Bounds check for indices
                            if idx_a < positions_vec.len()
                                && idx_b < positions_vec.len()
                                && idx_c < positions_vec.len()
                            {
                                let p_a = positions_vec[idx_a];
                                let p_b = positions_vec[idx_b];
                                let p_c = positions_vec[idx_c];

                                // Calculate face normal using the cross product
                                // This assumes a right-handed coordinate system and standard CCW winding for front faces.
                                // The vector order is important for correct normal direction.
                                let edge_ab = p_b - p_a;
                                let edge_ac = p_c - p_a;
                                let face_normal = edge_ab.cross(edge_ac); // Not normalized yet, magnitude is proportional to area

                                // Accumulate the un-normalized face normal for all three vertices
                                // This weights the vertex normal by the area of the contributing face.
                                calculated_normals[idx_a] += face_normal;
                                calculated_normals[idx_b] += face_normal;
                                calculated_normals[idx_c] += face_normal;
                            } else {
                                warn!("Index out of bounds during normal calculation at triangle starting index {}", i);
                            }
                        }
                    }

                    // Normalize the accumulated vertex normals to get the final unit vectors
                    for normal in calculated_normals.iter_mut() {
                        // Handle potential zero vectors (e.g., isolated vertex, degenerate faces)
                        let norm_magnitude = normal.magnitude();
                        if norm_magnitude > f32::EPSILON {
                            // Check for non-zero length before normalizing
                            *normal = *normal / norm_magnitude;
                        } else {
                            // If the normal remains zero, a default is needed.
                            // Keeping as zero for consistency with previous behavior,
                            // though a context-specific default (e.g., 'up') might be better.
                            // Log a trace message as it's usually not critical but good to know.
                            log::trace!(
                                "Calculated normal resulted in zero vector, keeping as zero."
                            );
                        }
                    }

                    calculated_normals
                }
            };
            // --- Normal Calculation Logic End ---

            // Collect other data needed for vertex creation
            let tangents_vec: Option<Vec<_>> = tangents.map(|t| t.collect());
            let uvs_vec: Option<Vec<_>> =
                uvs.map(|uv| uv.map(|uv| Vector2::new(uv[0], uv[1])).collect());

            // Detect if this is likely a UV sphere mesh for better tangent generation
            let is_uv_sphere = if tangents_vec.is_none() {
                if let Some(uvs) = &uvs_vec {
                    // Analyze UV coordinates to detect UV sphere pattern
                    let mut has_pole_vertices = false;
                    let mut has_equator_vertices = false;

                    for uv in uvs {
                        // Check for pole vertices (V = 0 or V = 1)
                        if uv.y < 0.01 || uv.y > 0.99 {
                            has_pole_vertices = true;
                        }
                        // Check for equator vertices (V ≈ 0.5)
                        if uv.y > 0.49 && uv.y < 0.51 {
                            has_equator_vertices = true;
                        }
                    }

                    // If we have both pole and equator vertices, it's likely a UV sphere
                    has_pole_vertices && has_equator_vertices
                } else {
                    // No UVs provided - check if this looks like a sphere by analyzing vertex positions
                    // A sphere should have vertices at various distances from center, with some at radius 1
                    let mut has_center_distance_vertices = false;
                    let mut has_unit_radius_vertices = false;

                    for position in &positions_vec {
                        let distance = position.magnitude();
                        if distance < 0.1 {
                            has_center_distance_vertices = true;
                        }
                        if (distance - 1.0).abs() < 0.1 {
                            has_unit_radius_vertices = true;
                        }
                    }

                    // If we have vertices at various distances and some at unit radius, likely a sphere
                    has_center_distance_vertices && has_unit_radius_vertices
                }
            } else {
                false
            };

            if is_uv_sphere {
                log::debug!("Detected UV sphere mesh - using sphere-specific tangent generation");
                log::debug!(
                    "Sphere has {} vertices, UVs provided: {}",
                    positions_vec.len(),
                    uvs_vec.is_some()
                );
            }

            // Main vertex processing loop
            let mut vertices = Vec::new();
            for (i, position) in positions_vec.iter().enumerate() {
                // No coordinate system conversion needed - engine uses Y-up like glTF
                let position = *position;

                // Use normal directly from glTF (no coordinate conversion needed)
                let normal_gltf = normals_vec.get(i).unwrap();
                let normal = *normal_gltf;

                // Read tangent with handedness (w component) properly
                let tangent_data = tangents_vec.as_ref().and_then(|tangents| tangents.get(i));

                let (tangent, bitangent) = if let Some(tangent_raw) = tangent_data {
                    // Use tangent coordinates directly from glTF (no coordinate conversion needed)
                    let tangent_vec = Vector3::new(tangent_raw[0], tangent_raw[1], tangent_raw[2]);
                    let handedness = tangent_raw[3]; // w component defines handedness

                    // Calculate bitangent using the (potentially calculated) normal and tangent with correct handedness
                    let calculated_bitangent = normal.cross(tangent_vec) * handedness;
                    (tangent_vec, calculated_bitangent)
                } else {
                    // When tangent is missing, try to detect if this is a sphere-like mesh
                    // and use appropriate tangent generation
                    let uv = uvs_vec
                        .as_ref()
                        .and_then(|uvs| uvs.get(i))
                        .map(|uv| (uv.x, uv.y));

                    // Use the detected UV sphere information for better tangent generation
                    if is_uv_sphere {
                        // Use sphere-specific tangent generation for better pole handling
                        log::trace!("Using sphere-specific tangent generation for vertex {}", i);
                        tangent_utils::generate_sphere_tangent_frame(normal, uv)
                    } else {
                        // Use the general arbitrary tangent frame generator
                        log::trace!("Using arbitrary tangent generation for vertex {}", i);
                        tangent_utils::generate_arbitrary_tangent_frame(normal)
                    }
                };

                let uv = if let Some(uvs) = &uvs_vec {
                    // Use original UV coordinates if available - these should be correct from Blender
                    if let Some(uv) = uvs.get(i) {
                        Vector2::new(uv.x, uv.y)
                    } else {
                        warn!("UV missing for vertex {i}. Using default!");
                        Vector2::zero()
                    }
                } else {
                    // No UVs provided at all - this shouldn't happen for proper glTF files
                    warn!("No UV coordinates provided for mesh. This may cause rendering issues.");
                    Vector2::zero()
                };

                // Create vertex with the calculated or provided normal, tangent, and bitangent
                let vertex = Vertex::new_with_bitangent(position, normal, tangent, bitangent, uv);
                vertices.push(vertex);
            }

            // Collect indices into a vector first
            let indices_vec: Vec<u32> = indices.collect();

            // Flip the winding order of indices to account for coordinate system handedness
            let mut indices_flipped = Vec::new();
            for i in (0..indices_vec.len()).step_by(3) {
                if i + 2 < indices_vec.len() {
                    // Flip the triangle winding order
                    indices_flipped.push(indices_vec[i]);
                    indices_flipped.push(indices_vec[i + 2]);
                    indices_flipped.push(indices_vec[i + 1]);
                }
            }

            let mesh_descriptor = MeshDescriptor {
                vertices,
                indices: indices_flipped,
            };
            let material = Self::parse_materials(&primitive.material(), textures);

            let decomposed = node.transform().decomposed();
            let transform = Transform {
                position: Vector3 {
                    x: decomposed.0[0],
                    y: decomposed.0[1],
                    z: decomposed.0[2],
                },
                rotation: {
                    Quaternion::new(
                        decomposed.1[1],
                        decomposed.1[0],
                        decomposed.1[2],
                        decomposed.1[3],
                    )
                },
                scale: Vector3 {
                    x: decomposed.2[0],
                    y: decomposed.2[1],
                    z: decomposed.2[2],
                },
            };

            let model = ModelDescriptor {
                label: node
                    .name()
                    .map(|x| x.to_string())
                    .unwrap_or("Unnamed".to_string()),
                mesh: Arc::new(mesh_descriptor),
                materials: vec![Arc::new(material)],
                transforms: vec![transform],
            };

            results.push(model);
        }

        Ok(results)
    }

    /// Handles parsing of glTF [`Camera`] and turns it into an Orbital [`CameraDescriptor`].
    fn parse_camera(
        node: &Node,
        camera: &Camera,
        buffers: &Vec<gltf::buffer::Data>,
    ) -> Result<CameraDescriptor, Box<dyn Error>> {
        let perspective = match camera.projection() {
            Projection::Orthographic(_) => {
                return Err(Box::new(GltfError::Unsupported));
            }
            Projection::Perspective(perspective) => perspective,
        };

        let transform = node.transform();
        let decomposed = transform.decomposed();

        let quaternion = Quaternion::new(
            decomposed.1[1],
            decomposed.1[0],
            decomposed.1[2],
            decomposed.1[3],
        );
        let (pitch, yaw) = quaternion_to_pitch_yaw(&quaternion);

        let camera_descriptor = CameraDescriptor {
            label: node
                .name()
                .map(|x| x.to_string())
                .unwrap_or("Unnamed".to_string()),
            position: Point3::new(decomposed.0[0], decomposed.0[1], decomposed.0[2]),
            yaw,
            pitch,
            aspect: perspective.aspect_ratio().unwrap_or(16.0 / 9.0),
            fovy: perspective.yfov(),
            near: perspective.znear(),
            far: perspective.znear(),
            global_gamma: CameraDescriptor::DEFAULT_GAMMA,
        };

        Ok(camera_descriptor)
    }

    /// Computes tangent and bitangent vectors based on UV gradients
    /// This is a simplified approach - a more robust implementation would use the mikktspace algorithm
    fn compute_tangent_frame(
        positions: &[Vector3<f32>],
        normals: &Option<Vec<Vector3<f32>>>,
        uvs: &Option<Vec<Vector2<f32>>>,
        index: usize,
    ) -> (Vector3<f32>, Vector3<f32>) {
        // For a single vertex, we can't compute proper tangents without looking at the full triangle
        // As a fallback, we'll create an arbitrary tangent that's orthogonal to the normal
        if let Some(normals_vec) = normals {
            if let Some(normal) = normals_vec.get(index) {
                // Create an arbitrary vector not parallel to the normal
                let arbitrary = if normal.x.abs() > 0.9 {
                    Vector3::new(0.0, 1.0, 0.0)
                } else {
                    Vector3::new(1.0, 0.0, 0.0)
                };

                // Compute tangent as orthogonal to normal
                let tangent = arbitrary.cross(*normal).normalize();
                // Compute bitangent as orthogonal to both
                let bitangent = normal.cross(tangent);

                return (tangent, bitangent);
            }
        }

        // Fallback to zero vectors
        (Vector3::zero(), Vector3::zero())
    }
}

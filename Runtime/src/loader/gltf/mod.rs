use crate::resources::{
    CameraDescriptor, FilterMode, MaterialDescriptor, MeshDescriptor,
    ModelDescriptor, PBRMaterialDescriptor, TextureDescriptor, TextureSize, Transform, Vertex,
};
use cgmath::{Point3, Quaternion, Vector2, Vector3, Zero};
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

        let byte_requirement = format.target_component_alignment().unwrap_or(1) as usize;
        let mut pixels =
            Vec::with_capacity(data.pixels.len() + (data.pixels.len() / (byte_requirement - 1)));
        if need_alpha_channel {
            for (i, pixel) in data.pixels.iter().enumerate() {
                if i % byte_requirement == 0 {
                    for _ in 0..byte_requirement {
                        pixels.push(0u8);
                    }
                }

                pixels.push(*pixel);
            }
        }

        TextureDescriptor::Data {
            pixels: data.pixels.clone(),
            size: TextureSize {
                width: data.width,
                height: data.height,
                depth_or_array_layers: 0,
                base_mip: 0,
                mip_levels: 0,
            },
            usages: TextureUsages::RENDER_ATTACHMENT,
            format,
            texture_dimension: TextureDimension::D1,
            texture_view_dimension: TextureViewDimension::D1,
            filter_mode: FilterMode::linear(),
        }
    }

    /// Handles parsing a "dual" texture.
    /// Same as [`Self::parse_texture`], but splits the R(ed) and G(reen) channel into two separate
    /// textures. Only supports R+G dual textures at the moment.
    fn parse_dual_texture(data: &gltf::image::Data) -> (TextureDescriptor, TextureDescriptor) {
        let (format, need_alpha_channel) = Self::gltf_texture_format_to_orbital(data.format);

        let byte_requirement = format.target_component_alignment().unwrap_or(1) as usize;
        let mut pixels_0 = Vec::with_capacity(data.pixels.len() / byte_requirement);
        let mut pixels_1 = Vec::with_capacity(data.pixels.len() / byte_requirement);

        for (i, pixel) in data.pixels.iter().enumerate() {
            if i % byte_requirement == 0 {
                pixels_0.push(*pixel);
            } else {
                pixels_1.push(*pixel);
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
                depth_or_array_layers: 0,
                base_mip: 0,
                mip_levels: 0,
            },
            usages: TextureUsages::RENDER_ATTACHMENT,
            format: actual_format,
            texture_dimension: TextureDimension::D1,
            texture_view_dimension: TextureViewDimension::D1,
            filter_mode: FilterMode::linear(),
        };
        let texture_1 = TextureDescriptor::Data {
            pixels: pixels_1,
            size: TextureSize {
                width: data.width,
                height: data.height,
                depth_or_array_layers: 0,
                base_mip: 0,
                mip_levels: 0,
            },
            usages: TextureUsages::RENDER_ATTACHMENT,
            format: actual_format,
            texture_dimension: TextureDimension::D1,
            texture_view_dimension: TextureViewDimension::D1,
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
            TextureDescriptor::uniform_luma_black()
        };

        let albedo =
            if let Some(albedo_info) = material.pbr_metallic_roughness().base_color_texture() {
                Self::parse_texture(&textures[albedo_info.texture().source().index()])
            } else {
                TextureDescriptor::uniform_rgba_color(Color {
                    r: 0.5,
                    g: 0.0,
                    b: 0.5,
                    a: 1.0,
                })
            };
        let albedo_factor_raw = material.pbr_metallic_roughness().base_color_factor();
        // Note: Skipping 'w' here!
        let albedo_factor = Vector3::new(
            albedo_factor_raw[0],
            albedo_factor_raw[1],
            albedo_factor_raw[2],
        );

        let (metallic, roughness) = if let Some(metallic_and_roughness_info) = material
            .pbr_metallic_roughness()
            .metallic_roughness_texture()
        {
            Self::parse_dual_texture(
                &textures[metallic_and_roughness_info.texture().source().index()],
            )
        } else {
            (
                TextureDescriptor::uniform_rgba_color(Color {
                    r: 0.5,
                    g: 0.0,
                    b: 0.5,
                    a: 1.0,
                }),
                TextureDescriptor::uniform_rgba_color(Color {
                    r: 0.5,
                    g: 0.0,
                    b: 0.5,
                    a: 1.0,
                }),
            )
        };
        let metallic_factor = material.pbr_metallic_roughness().metallic_factor();
        let roughness_factor = material.pbr_metallic_roughness().roughness_factor();

        let occlusion = if let Some(occlusion_info) = material.occlusion_texture() {
            Self::parse_texture(&textures[occlusion_info.texture().source().index()])
        } else {
            TextureDescriptor::uniform_rgba_black()
        };
        let emissive = if let Some(emissive_info) = material.emissive_texture() {
            Self::parse_texture(&textures[emissive_info.texture().source().index()])
        } else {
            let emissive_color = material.emissive_factor();
            TextureDescriptor::uniform_rgba_color(Color {
                r: emissive_color[0] as f64,
                g: emissive_color[1] as f64,
                b: emissive_color[2] as f64,
                a: 1.0,
            })
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

            let mut vertices = Vec::new();
            for (i, position_raw) in positions.enumerate() {
                let position = Vector3::new(position_raw[0], position_raw[1], position_raw[2]);

                let normal = normals
                    .as_mut()
                    .and_then(|iter| iter.nth(i))
                    .map(|n| Vector3::new(n[0], n[1], n[2]))
                    .unwrap_or_else(|| {
                        warn!("Normal missing for vertex {i}. Using default!");
                        Vector3::zero()
                    });

                // Note: `w` is being ignored here!
                let tangent = tangents
                    .as_mut()
                    .and_then(|iter| iter.nth(i))
                    .map(|n| Vector3::new(n[0], n[1], n[2]))
                    .unwrap_or_else(|| {
                        warn!("Tangent missing for vertex {i}. Using default!");
                        Vector3::zero()
                    });

                let uv = uvs
                    .as_mut()
                    .and_then(|iter| iter.nth(i))
                    .map(|n| Vector2::new(n[0], n[1]))
                    .unwrap_or_else(|| {
                        warn!("Tangent missing for vertex {i}. Using default!");
                        Vector2::zero()
                    });

                let vertex = Vertex::new(position, normal, tangent, uv);
                vertices.push(vertex);
            }

            let mesh_descriptor = MeshDescriptor {
                vertices,
                indices: indices.collect(),
            };
            let material = Self::parse_materials(&primitive.material(), textures);

            let decomposed = node.transform().decomposed();
            let transform = Transform {
                position: Vector3 {
                    x: decomposed.0[0],
                    y: decomposed.0[1],
                    z: decomposed.0[2],
                },
                rotation: Quaternion::new(
                    decomposed.1[0],
                    decomposed.1[1],
                    decomposed.1[2],
                    decomposed.1[3],
                ),
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
            decomposed.1[0],
            decomposed.1[1],
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
}

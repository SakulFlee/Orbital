use std::error::Error;
use std::{fs, io};
use std::path::Path;
use std::sync::Arc;
use cgmath::{Deg, Euler, Point3, Quaternion, Vector2, Vector3, Zero};
use gltf::camera::Projection;
use gltf::{Attribute, Buffer, Camera, Document, Gltf, Material, Mesh, Node, Semantic};
use gltf::image::{Data, Format};
use gltf::texture::Info;
use log::{debug, warn};
use wgpu::{Color, TextureDimension, TextureFormat, TextureUsages, TextureViewDimension};
use crate::resources::{CameraDescriptor, FilterMode, MaterialDescriptor, MaterialShader, MeshDescriptor, ModelDescriptor, PBRMaterialDescriptor, TextureDescriptor, TextureSize, Transform, Vertex};

mod import;
pub use import::*;

mod task;
pub use task::*;

mod import_type;
pub use import_type::*;

mod result;
pub use result::*;

mod error;
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
pub struct GltfLoader;

impl GltfLoader {

    /// TODO
    /// TODO: Add note about instancing!
    pub async fn load(import_task: GltfImportTask) -> Result<GltfImportResult, Box<dyn Error>> {
        let (document, buffers, textures) = gltf::import(&import_task.file).map_err(|e| Box::new(e))?;

            let mut i = 0;
        for scene in document.scenes() {
            for node in scene.nodes() {
                debug!("");
                debug!("#{}: {:?}", i, &node);
                i += 1;

                if let Some(camera) = node.camera() {
                    debug!("Is camera!");
                    debug!("Name: {:?}", node.name());
                    debug!("Projection: {:?}", camera.projection());
                    debug!("Transform: {:?}", node.transform());
                } else if let Some(mesh) = node.mesh() {
                    debug!("Is mesh!");
                    debug!("Name: {:?}", node.name());
                    debug!("Transform: {:?}", node.transform());
                    debug!("Primitives count: {}", mesh.primitives().len());

                    let mut a = 0;
                    for primitive in mesh.primitives() {
                        debug!("#{i}-{a} Bounding Box: {:?}", primitive.bounding_box());
                        debug!("#{i}-{a} Indices: {:?}", primitive.indices());
                        debug!("#{i}-{a} Material: {:?}", primitive.material());

                        let primitive_reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

                        debug!("#{i}-{a} Attribute count: {:?}", primitive.attributes().len());
                        let mut b = 0;
                        for attribute in primitive.attributes() {
                            match attribute.0 {
                                Semantic::Positions => {
                                    debug!("#{i}-{a}-{b} Position: {:?}", attribute.1);
                                    debug!("#{i}-{a}-{b} Data Type: {:?}", attribute.1.data_type());
                                    debug!("#{i}-{a}-{b} Normalized: {:?}", attribute.1.normalized());
                                    debug!("#{i}-{a}-{b} Count: {:?}", attribute.1.count());
                                    debug!("#{i}-{a}-{b} index: {:?}", attribute.1.index());
                                    debug!("#{i}-{a}-{b} view: {:?}", attribute.1.view());

                                    if let Some(positions_iter) = primitive_reader.read_positions() {
                                        for (idx, position) in positions_iter.enumerate() {
                                            debug!("#{} {:?}", idx, position);
                                        }
                                    }
                                }
                                _ => {warn!("Unimplemented sematic: {:?}", attribute.0)},
                            }

                            b+= 1;
                        }

                        debug!("#{i}-{a} Morph Targets: {:?}", primitive.morph_targets());
                        let mut b = 0;
                        for target in primitive.morph_targets() {
                            debug!("#{i}-{a}-{b} Positions: {:?}", target.positions());
                            debug!("#{i}-{a}-{b} Normals: {:?}", target.normals());
                            debug!("#{i}-{a}-{b} Tangents: {:?}", target.tangents());

                            b += 1;
                        }

                        a+= 1;
                    }
                } else {
                    debug!("Is unknown!");
                    debug!("Name: {:?}", node.name());
                }
            }
        }

        todo!()
    }

    /// Returns the Orbital/WGPU [`TextureFormat`] as well as a boolean.
    /// If the boolean is true, that means an alpha channel has to be generated.
    /// This is due to WGPU not supporting RGB textures without an alpha channel.
    fn gltf_texture_format_to_orbital(format: Format) -> (TextureFormat, bool) {
        match format {
            Format::R8 => {(TextureFormat::R8Unorm, false)}
            Format::R8G8 => {(TextureFormat::Rg8Unorm, false)}
            Format::R8G8B8 => {(TextureFormat::Rgba8Unorm, true)}
            Format::R8G8B8A8 => {(TextureFormat::Rgba8Unorm, false)}
            Format::R16 => {(TextureFormat::R16Unorm, false)}
            Format::R16G16 => {(TextureFormat::Rg16Unorm, false)}
            Format::R16G16B16 => {(TextureFormat::Rgba16Unorm, true)}
            Format::R16G16B16A16 => {(TextureFormat::Rgba16Unorm, false)}
            Format::R32G32B32FLOAT => {(TextureFormat::Rgba32Float, true)}
            Format::R32G32B32A32FLOAT => {(TextureFormat::Rgba32Float, false)}
        }
    }

    fn parse_texture(data: &gltf::image::Data) -> TextureDescriptor {
        let (format, need_alpha_channel) = Self::gltf_texture_format_to_orbital(data.format);

        let byte_requirement = format.target_component_alignment().unwrap_or(1) as usize;
        let mut pixels = Vec::with_capacity(data.pixels.len() + (data.pixels.len() / (byte_requirement - 1)));
        if need_alpha_channel{
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
            format, // TODO Should be R8/16/32 based on actual format, not split/dual-formats!
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
            format,
            texture_dimension: TextureDimension::D1,
            texture_view_dimension: TextureViewDimension::D1,
            filter_mode: FilterMode::linear(),
        };

        (texture_0, texture_1)
    }

    fn parse_materials(material: &Material, textures: &Vec<gltf::image::Data>) -> MaterialDescriptor {
        let normal = if let Some(normal_info) = material.normal_texture() {
            Self::parse_texture(&textures[normal_info.texture().source().index()]) } else {TextureDescriptor::uniform_luma_black()};

        let albedo = if let Some(albedo_info) = material.pbr_metallic_roughness().base_color_texture() {Self::parse_texture(&textures[albedo_info.texture().source().index()]) } else {TextureDescriptor::uniform_rgba_color(Color { r: 0.5, g: 0.0, b: 0.5, a: 1.0})};
        let albedo_factor_raw = material.pbr_metallic_roughness().base_color_factor();
        // Note: Skipping 'w' here!
        let albedo_factor = Vector3::new(albedo_factor_raw[0], albedo_factor_raw[1], albedo_factor_raw[2]);

        let (metallic, roughness) = if let Some(metallic_and_roughness_info) = material.pbr_metallic_roughness().metallic_roughness_texture() {Self::parse_dual_texture(&textures[metallic_and_roughness_info.texture().source().index()]) } else {
            (
                TextureDescriptor::uniform_rgba_color(Color { r: 0.5, g: 0.0, b: 0.5, a: 1.0}),
                TextureDescriptor::uniform_rgba_color(Color { r: 0.5, g: 0.0, b: 0.5, a: 1.0})
            )
        };
        let metallic_factor = material.pbr_metallic_roughness().metallic_factor();
        let roughness_factor = material.pbr_metallic_roughness().roughness_factor();

        let occlusion = if let Some(occlusion_info) = material.occlusion_texture() {Self::parse_texture(&textures[occlusion_info.texture().source().index()]) } else {TextureDescriptor::uniform_rgba_black()};
        let emissive = if let Some(emissive_info) = material.emissive_texture() {Self::parse_texture(&textures[emissive_info.texture().source().index()]) } else {
            let emissive_color = material.emissive_factor();
            TextureDescriptor::uniform_rgba_color(Color {
                r: emissive_color[0] as f64,
                g: emissive_color[1] as f64,
                b: emissive_color[2] as f64,
                a: 1.0,
            })};

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

    fn parse_models(node: &Node, mesh: &Mesh, buffers: &Vec<gltf::buffer::Data>, textures: &Vec<gltf::image::Data>) -> Result<Vec<ModelDescriptor>, Box<dyn Error>> {
        let primitives = mesh.primitives();
        let mut results = Vec::new();

        // glTF Primitive == Orbital Model
        for primitive in primitives {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            let Some(positions) = reader.read_positions() else {
                warn!("Primitive has no positions. Skipping mesh primitive.");
                continue;
            };
            let Some(indices) = reader.read_indices().and_then(|x| Some(x.into_u32())) else {
                warn!("Primitive has no indices. Skipping mesh primitive.");
                continue;
            };
            let mut normals = reader.read_normals();
            let mut tangents = reader.read_tangents();
            let mut uvs = reader.read_tex_coords(0).and_then(|x| Some(x.into_f32()));
            primitive.attributes().for_each(|x| if let Semantic::TexCoords(indices) = x.0 {
                if indices > 1 {
                    warn!("More than one UV index found, only the first will be imported!");
                }
            });

            let mut vertices = Vec::new();
            for (i, position_raw) in positions.enumerate() {
                let position = Vector3::new(position_raw[0], position_raw[1], position_raw[2]);

                let normal = normals.as_mut().and_then(|iter| iter.nth(i))
                    .map(|n| Vector3::new(n[0], n[1], n[2]))
                    .unwrap_or_else(|| {
                        warn!("Normal missing for vertex {}. Using default!", i);
                        Vector3::zero()
                    });

                // Note: `w` is being ignored here!
                let tangent = tangents.as_mut().and_then(|iter| iter.nth(i))
                    .map(|n| Vector3::new(n[0], n[1], n[2]))
                    .unwrap_or_else(|| {
                        warn!("Tangent missing for vertex {}. Using default!", i);
                        Vector3::zero()
                    });

                let uv = uvs.as_mut().and_then(|iter| iter.nth(i))
                    .map(|n| Vector2::new(n[0], n[1]))
                    .unwrap_or_else(|| {
                        warn!("Tangent missing for vertex {}. Using default!", i);
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
            let transform =Transform {
                position: Vector3 {
                    x: decomposed.0[0],
                    y: decomposed.0[1],
                    z: decomposed.0[2],
                },
                rotation: Quaternion::from(
                    Euler::new(
                    Deg(decomposed.1[0]),
                    Deg(decomposed.1[1]),
                    Deg(decomposed.1[2]),
                    )
                ),
                scale: Vector3 {
                    x: decomposed.2[0],
                    y: decomposed.2[1],
                    z: decomposed.2[2],
                },
            } ;

            let model = ModelDescriptor {
                label: mesh.name().map(|x| x.to_string()).unwrap_or("Unnamed".to_string()),
                mesh: Arc::new(mesh_descriptor),
                materials: vec![Arc::new(material)],
                transforms: vec![transform],
            };

            results.push(model);
        }

        Ok(results)
    }

    fn parse_camera(node: &Node, camera: &Camera, buffers: &Vec<gltf::buffer::Data>) -> Result<CameraDescriptor, Box<dyn Error>> {
        let perspective = match camera.projection() {
            Projection::Orthographic(_) => {
                return Err(Box::new(GltfError::Unsupported));
            }
            Projection::Perspective(perspective) => {
                perspective
            }
        };

        let transform = node.transform();
        let decomposed = transform.decomposed();

        let camera_descriptor = CameraDescriptor {
            label: node.name().map(|x| x.to_string()).unwrap_or("Unnamed".to_string()),
            position: Point3::new(decomposed.0[0], decomposed.0[2], decomposed.0[2]),
            yaw: decomposed.1[0],
            pitch: decomposed.1[1],
            aspect: perspective.aspect_ratio().unwrap_or(16.0/9.0),
            fovy: perspective.yfov(),
            near: perspective.znear(),
            far: perspective.znear(),
            global_gamma: CameraDescriptor::DEFAULT_GAMMA,
        };

        Ok(camera_descriptor)
    }
}

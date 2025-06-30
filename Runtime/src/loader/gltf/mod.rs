use std::error::Error;
use std::{fs, io};
use std::path::Path;
use cgmath::{Vector2, Vector3, Zero};
use gltf::camera::Projection;
use gltf::{Attribute, Buffer, Document, Gltf, Node, Semantic};
use gltf::image::Data;
use log::{debug, warn};

use crate::resources::{MeshDescriptor, Vertex};

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

    fn parse_meshes(mesh_node: &Node, buffers: &Vec<gltf::buffer::Data>, textures: &Vec<gltf::image::Data>) -> Result<Vec<MeshDescriptor>, Box<dyn Error>> {
        let mesh = mesh_node.mesh().ok_or(Box::new(GltfError::NodeNotMesh))?;
        let primitives = mesh.primitives();
        let mut results = Vec::new();

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

            let descriptor = MeshDescriptor {
                vertices,
                indices: indices.collect(),
            };
            results.push(descriptor);
        }

        Ok(results)
    }
}

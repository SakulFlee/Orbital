use std::borrow::Borrow;

use gltf::{
    accessor::Iter,
    image::Source,
    mesh::util::{ReadIndices, ReadTexCoords},
    Texture,
};
use nalgebra::{Vector2, Vector3};
use russimp::scene::{PostProcess, Scene};
use wgpu::{core::resource::Texture, Device, Queue, TextureFormat};

use crate::{
    error::Error,
    resources::{MaterialDescriptor, MeshDescriptor, ModelDescriptor, Vertex, VertexUniform},
};

use super::{Material, Mesh};

pub struct Model {
    mesh: Mesh,
    material: Material,
}

impl Model {
    pub fn from_descriptor(
        descriptor: &ModelDescriptor,
        surface_format: &TextureFormat,
        device: &Device,
        queue: &Queue,
    ) -> Result<Self, Error> {
        match descriptor {
            ModelDescriptor::FromDescriptors(mesh_descriptor, material_descriptor) => {
                Self::from_descriptors(
                    mesh_descriptor,
                    material_descriptor,
                    surface_format,
                    device,
                    queue,
                )
            }
            ModelDescriptor::FromFile(_) => todo!(),
            ModelDescriptor::FromBytes(bytes, hint) => Self::from_bytes(bytes, hint),
        }
    }

    pub fn from_descriptors(
        mesh_descriptor: &MeshDescriptor,
        material_descriptor: &MaterialDescriptor,
        surface_format: &TextureFormat,
        device: &Device,
        queue: &Queue,
    ) -> Result<Self, Error> {
        let mesh = Mesh::from_descriptor(mesh_descriptor, device, queue);

        let material =
            Material::from_descriptor(material_descriptor, surface_format, device, queue)?;

        Ok(Self::from_existing(mesh, material))
    }

    pub fn from_bytes(
        bytes: &[u8],
        hint: &'static str,
        device: &Device,
    ) -> Result<Vec<Self>, Error> {
        let (document, buffers, images) =
            gltf::import_slice(bytes).map_err(|e| Error::GltfError(e))?;

            for buffer in document.buffers() {
                match buffer.source() {
                    gltf::buffer::Source::Bin => todo!(),
                    gltf::buffer::Source::Uri(_) => todo!(),
                }
            }

        let mut material_result = Vec::<Material>::new();
        for gltf_material in document.materials() {
            let pbr = gltf_material.pbr_metallic_roughness();

            let albedo = pbr.base_color_texture();

            let tex = albedo.unwrap().texture();
            let img = tex.source();
            let src = img.source();

            // TODO
            match src {
                Source::View { view, mime_type } => {
                    let buffer = view.buffer();
                    let src = buffer.source();
                    match src {
                        gltf::buffer::Source::Bin => {
                            let img = images[buffer.index()];
                            let pxl = img.pixels;
                            let width = img.width;
                            let height = img.height;
                            let format = img.format;
                        }
                        gltf::buffer::Source::Uri(_) => todo!(),
                    }
                }
                Source::Uri { uri, mime_type } => todo!(),
            }

            let albedo_factor = pbr.base_color_factor();
            let metallic = pbr.metallic_roughness_texture();
            let metallic_factor = pbr.metallic_factor();
            let roughness_factor = pbr.roughness_factor();
        }

        let mut mesh_result = Vec::<Mesh>::new();
        for gltf_mesh in document.meshes() {
            mesh_result.push(Mesh::from_gltf(gltf_mesh, buffers, device));
        }

        // TODO: Materials
        // TODO: From File
        // TODO: By name?

        todo!()
    }

    pub fn from_existing(mesh: Mesh, material: Material) -> Self {
        Self { mesh, material }
    }

    pub fn mesh(&self) -> &Mesh {
        &self.mesh
    }

    pub fn material(&self) -> &Material {
        &self.material
    }
}

use std::borrow::Borrow;

use gltf::{
    accessor::Iter,
    mesh::util::{ReadIndices, ReadTexCoords},
};
use nalgebra::{Vector2, Vector3};
use russimp::scene::{PostProcess, Scene};
use wgpu::{Device, Queue, TextureFormat};

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

        let mut result = Vec::<Mesh>::new();
        for gltf_mesh in document.meshes() {
            result.push(Mesh::from_gltf(gltf_mesh, buffers, device));
        }

        // TODO: Materials
        // TODO: From File

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

use wgpu::{Device, Queue, TextureFormat};

use crate::{
    error::Error,
    resources::{ImportDescriptor, MaterialDescriptor, MeshDescriptor, ModelDescriptor},
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
            #[cfg(feature = "gltf")]
            ModelDescriptor::FromGLTF(file, scene_import_descriptor, model_import_descriptor) => {
                Self::from_gltf(
                    file,
                    scene_import_descriptor,
                    model_import_descriptor,
                    surface_format,
                    device,
                    queue,
                )
            }
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

    #[cfg(feature = "gltf")]
    pub fn from_gltf(
        file: &'static str,
        scene_import_descriptor: &ImportDescriptor,
        model_import_descriptor: &ImportDescriptor,
        surface_format: &TextureFormat,
        device: &Device,
        queue: &Queue,
    ) -> Result<Self, Error> {
        // Load glTF file
        let gltf_file = easy_gltf::load(file).map_err(|e| Error::GltfError(e))?;

        // Query for scene. If found we continue.
        let scene = if let Some(scene) = match scene_import_descriptor {
            ImportDescriptor::Index(i) => gltf_file.get(*i as usize),
            ImportDescriptor::Name(name) => gltf_file
                .iter()
                .find(|x| x.name.is_some() && x.name.as_ref().unwrap() == *name),
        } {
            scene
        } else {
            return Err(Error::SceneNotFound);
        };

        // Query for model. If found we continue.
        let models = &scene.models;
        let model = if let Some(model) = match model_import_descriptor {
            ImportDescriptor::Index(i) => models.get(*i as usize),
            ImportDescriptor::Name(name) => models.iter().find(|x| {
                let mesh_name = x.mesh_name();

                mesh_name.is_some() && mesh_name.unwrap() == *name
            }),
        } {
            model
        } else {
            return Err(Error::ModelNotFound);
        };

        // Realize model
        let model = Self::from_existing(
            Mesh::from_gltf(&model, device)?,
            Material::from_gltf(&model.material(), surface_format, device, queue)?,
        );

        Ok(model)
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

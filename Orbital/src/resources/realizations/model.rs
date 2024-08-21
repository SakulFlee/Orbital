use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages, Device, Queue, TextureFormat,
};

use crate::{
    error::Error,
    resources::descriptors::{
        ImportDescriptor, Instancing, MaterialDescriptor, MeshDescriptor, ModelDescriptor,
    },
};

use super::{instance::Instance, Material, Mesh};

#[derive(Debug)]
pub struct Model {
    mesh: Mesh,
    material_descriptor: MaterialDescriptor,
    instances: Vec<Instance>,
    instance_buffer: Buffer,
}

impl Model {
    pub fn from_descriptor(
        descriptor: &ModelDescriptor,
        device: &Device,
        queue: &Queue,
    ) -> Result<Self, Error> {
        match descriptor {
            ModelDescriptor::FromDescriptors(mesh_descriptor, material_descriptor, instancing) => {
                Self::from_descriptors(
                    mesh_descriptor,
                    material_descriptor,
                    instancing,
                    device,
                    queue,
                )
            }
            #[cfg(feature = "gltf")]
            ModelDescriptor::FromGLTF(
                file,
                scene_import_descriptor,
                model_import_descriptor,
                instancing,
            ) => Self::from_gltf(
                file,
                scene_import_descriptor,
                model_import_descriptor,
                instancing,
                device,
                queue,
            ),
        }
    }

    pub fn from_descriptors(
        mesh_descriptor: &MeshDescriptor,
        material_descriptor: &MaterialDescriptor,
        instancing: &Instancing,
        device: &Device,
        queue: &Queue,
    ) -> Result<Self, Error> {
        let mesh = Mesh::from_descriptor(mesh_descriptor, device, queue);

        let instances = Self::convert_instancing(instancing);

        Ok(Self::from_existing(
            mesh,
            material_descriptor.clone(),
            instances,
            device,
            queue,
        ))
    }

    #[cfg(feature = "gltf")]
    pub fn from_gltf(
        file: &'static str,
        scene_import_descriptor: &ImportDescriptor,
        model_import_descriptor: &ImportDescriptor,
        instancing: &Instancing,
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

        Self::from_gltf_model(
            model,
            Self::convert_instancing(instancing),
            device,
            queue,
        )
    }

    #[cfg(feature = "gltf")]
    pub fn from_gltf_model(
        model: &easy_gltf::Model,
        instances: Vec<Instance>,
        device: &Device,
        queue: &Queue,
    ) -> Result<Self, Error> {
        let material_descriptor: MaterialDescriptor = model.material().as_ref().into();

        Ok(Self::from_existing(
            Mesh::from_gltf(model, device)?,
            material_descriptor,
            instances,
            device,
            queue,
        ))
    }

    pub fn from_existing(
        mesh: Mesh,
        material_descriptor: MaterialDescriptor,
        instances: Vec<Instance>,
        device: &Device,
        queue: &Queue,
    ) -> Self {
        let instance_buffer = Self::make_instance_buffer(&instances, device, queue);

        Self {
            mesh,
            material_descriptor,
            instances,
            instance_buffer,
        }
    }

    fn make_instance_buffer(instances: &[Instance], device: &Device, _queue: &Queue) -> Buffer {
        let instance_data: Vec<u8> = instances
            .iter()
            .map(|x| x.make_model_space_matrix())
            .flat_map(|x| {
                vec![
                    x.x.x.to_le_bytes(),
                    x.x.y.to_le_bytes(),
                    x.x.z.to_le_bytes(),
                    x.x.w.to_le_bytes(),
                    x.y.x.to_le_bytes(),
                    x.y.y.to_le_bytes(),
                    x.y.z.to_le_bytes(),
                    x.y.w.to_le_bytes(),
                    x.z.x.to_le_bytes(),
                    x.z.y.to_le_bytes(),
                    x.z.z.to_le_bytes(),
                    x.z.w.to_le_bytes(),
                    x.w.x.to_le_bytes(),
                    x.w.y.to_le_bytes(),
                    x.w.z.to_le_bytes(),
                    x.w.w.to_le_bytes(),
                ]
            })
            .flatten()
            .collect();

        device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: &instance_data,
            usage: BufferUsages::VERTEX,
        })
    }

    pub fn convert_instancing(instancing: &Instancing) -> Vec<Instance> {
        match instancing {
            Instancing::Single(i) => vec![Instance::from_descriptor(i)],
            Instancing::Multiple(vi) => vi.iter().map(Instance::from_descriptor).collect(),
        }
    }

    pub fn update_instance_buffer(&mut self, device: &Device, queue: &Queue) {
        self.instance_buffer = Self::make_instance_buffer(&self.instances, device, queue);
    }

    pub fn mesh(&self) -> &Mesh {
        &self.mesh
    }

    pub fn material(
        &self,
        surface_format: &TextureFormat,
        device: &Device,
        queue: &Queue,
    ) -> Result<&'static Material, Error> {
        Material::from_descriptor(&self.material_descriptor, surface_format, device, queue)
    }

    pub fn instances(&self) -> &Vec<Instance> {
        &self.instances
    }

    pub fn instance_buffer(&self) -> &Buffer {
        &self.instance_buffer
    }
}

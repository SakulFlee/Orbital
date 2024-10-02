use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages, Device, Queue, TextureFormat,
};

use crate::{
    resources::descriptors::{MaterialDescriptor, MeshDescriptor, ModelDescriptor},
    transform::Transform,
};

use super::{instance::Instance, Material, Mesh};

// TODO: Move out
#[derive(Debug)]
pub struct Instancing {
    buffer: Buffer,
    count: u32,
}

impl Instancing {
    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn instance_count(&self) -> u32 {
        self.count
    }
}

#[derive(Debug)]
pub struct Model {
    descriptor: ModelDescriptor,
    cached_mesh: Option<Mesh>,
    instance_data: Option<Instancing>,
}

impl Model {
    pub fn from_descriptor(descriptor: ModelDescriptor) -> Self {
        Self {
            descriptor,
            cached_mesh: None,
            instance_data: None,
        }
    }

    pub fn descriptor(&self) -> &ModelDescriptor {
        &self.descriptor
    }

    pub fn mesh_descriptor(&self) -> &MeshDescriptor {
        &self.descriptor().mesh
    }

    pub fn mesh(&self) -> &Mesh {
        self.cached_mesh.as_ref().unwrap()
    }

    pub fn material_descriptor(&self) -> &MaterialDescriptor {
        &self.descriptor().material
    }

    pub fn material(
        &self,
        surface_format: &TextureFormat,
        device: &Device,
        queue: &Queue,
    ) -> &Material {
        Material::from_descriptor(self.material_descriptor(), surface_format, device, queue)
            .expect("Material build failed")
    }

    pub fn label(&self) -> &String {
        &self.descriptor.label
    }

    fn convert_transforms_to_instances(&self) -> Vec<Instance> {
        self.descriptor
            .transforms
            .iter()
            .map(Instance::from)
            .collect()
    }

    pub fn transform(&self) -> &Transform {
        self
            .descriptor
            .transforms
            .first()
            .expect("At least one Transform must be present!")
    }

    pub fn transform_specific(&self, id: usize) -> &Transform {
        &self.descriptor.transforms[id]
    }

    pub fn transform_count(&self) -> usize {
        self.descriptor.transforms.len()
    }

    /// Sets one or multiple [Transform]s for this [Model].
    /// Will **replace** _any_ [Transform]s with the given [Transform]s.
    ///
    /// If this [Model] has multiple [Instance]s defined, all will be
    /// effectively removed with this.
    pub fn set_transforms(&mut self, transforms: Vec<Transform>) {
        self.descriptor.transforms = transforms;

        // Reset instancing information to trigger a rebuild on next preparation
        // cycle.
        self.instance_data = None;
    }

    /// Sets a specific [Transform] on this [Model].
    /// Will **replace** the selected [Transform] with the given [Transform],
    /// if found.
    pub fn set_specific_transform(&mut self, transform: Transform, index: usize) {
        if let Some(model_transform) = self.descriptor.transforms.get_mut(index) {
            *model_transform = transform;

            // Reset instancing information to trigger a rebuild on next preparation
            // cycle.
            self.instance_data = None;
        }
    }

    /// Adds one or many [Transform]_s_ to the [Model].
    /// Effectively, instancing the [Model].
    pub fn add_transforms(&mut self, transforms: Vec<Transform>) {
        self.descriptor.transforms.extend(transforms);

        // Reset instancing information to trigger a rebuild on next preparation
        // cycle.
        self.instance_data = None;
    }

    /// Removes a [Transform] from the [Model].
    ///
    /// ⚠️ Make sure at least one [Transform] is present!
    pub fn remove_transforms(&mut self, indices: Vec<usize>) {
        let transform_drain = self.descriptor.transforms.drain(..);

        self.descriptor.transforms = transform_drain
            .into_iter()
            .enumerate()
            .filter_map(|(i, transform)| {
                if indices.contains(&i) {
                    None
                } else {
                    Some(transform)
                }
            })
            .collect();

        // Reset instancing information to trigger a rebuild on next preparation
        // cycle.
        self.instance_data = None;
    }

    /// Applies the given [Transform] to the [Model].
    /// _All_ defined [Transform]s will be offset by the given
    /// [Transform].
    pub fn apply_transform(&mut self, transform: Transform) {
        self.descriptor.transforms.iter_mut().for_each(|x| {
            x.apply_transform(transform);
        });

        // Reset instancing information to trigger a rebuild on next preparation
        // cycle.
        self.instance_data = None;
    }

    /// Applies the given [Transform] to the [Model] given a specific index for
    /// the [Transform] selection.
    /// _Only_ the defined [Transform] will be offset by the given
    /// [Transform].
    pub fn apply_transform_specific(&mut self, transform: Transform, index: usize) {
        if let Some(model_transform) = self.descriptor.transforms.get_mut(index) {
            *model_transform = transform;

            // Reset instancing information to trigger a rebuild on next preparation
            // cycle.
            self.instance_data = None;
        }
    }

    pub fn instance_data(&self) -> &Instancing {
        self.instance_data.as_ref().unwrap()
    }

    pub fn prepare_render(&mut self, device: &Device, queue: &Queue) {
        self.prepare_mesh(device, queue);
        self.prepare_instance_data(device);
    }

    fn prepare_mesh(&mut self, device: &Device, queue: &Queue) {
        if self.cached_mesh.is_some() {
            return;
        }

        // If the mesh doesn't exist yet, create it and then return it.
        let mesh = Mesh::from_descriptor(self.mesh_descriptor(), device, queue);
        self.cached_mesh = Some(mesh);
    }

    fn prepare_instance_data(&mut self, device: &Device) {
        if self.instance_data.is_some() {
            return;
        }

        let instances = self.convert_transforms_to_instances();
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
        let instance_count = self.transform_count() as u32;

        let instance_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: &instance_data,
            usage: BufferUsages::VERTEX,
        });

        // If the mesh doesn't exist yet, create it and then return it.
        let instance_data = Instancing {
            buffer: instance_buffer,
            count: instance_count,
        };
        self.instance_data = Some(instance_data);
    }
}

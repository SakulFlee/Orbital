use wgpu::{Device, Queue};

use crate::resources::ModelDescriptor;

use super::{Material, Mesh};

pub struct Model {
    mesh: Mesh,
    material: Material,
}

impl Model {
    pub fn from_descriptor(descriptor: &ModelDescriptor, device: &Device, queue: &Queue) -> Self {
        let mesh = Mesh::from_descriptor(&descriptor.mesh_descriptor, device, queue);

        let material = Material::from_descriptor(&descriptor.material_descriptor, device, queue);

        Self { mesh, material }
    }

    pub fn mesh(&self) -> &Mesh {
        &self.mesh
    }

    pub fn material(&self) -> &Material {
        &self.material
    }
}

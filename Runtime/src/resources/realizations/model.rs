use crate::{resources::ModelDescriptor, runtime::context::Context};

use super::{Material, Mesh};

pub struct Model {
    mesh: Mesh,
    material: Material,
}

impl Model {
    pub fn from_descriptor(descriptor: &ModelDescriptor, context: &Context) -> Self {
        let mesh = Mesh::from_descriptor(&descriptor.mesh_descriptor, context);

        let material = Material::from_descriptor(&descriptor.material_descriptor, context);

        Self { mesh, material }
    }

    pub fn mesh(&self) -> &Mesh {
        &self.mesh
    }

    pub fn material(&self) -> &Material {
        &self.material
    }
}

use super::{Material, Mesh};

pub struct Model {
    mesh: Mesh,
    material: Box<dyn Material>,
}

impl Model {
    pub fn new(mesh: Mesh, material: Box<dyn Material>) -> Self {
        Self { mesh, material }
    }

    pub fn mesh(&self) -> &Mesh {
        &self.mesh
    }

    pub fn material(&self) -> &Box<dyn Material> {
        &self.material
    }
}

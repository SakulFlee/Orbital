use orbital_mesh::MeshDescriptor;

pub trait Shape: Send + Sync {
    fn generate(&self) -> MeshDescriptor;
    fn name(&self) -> &str;
}

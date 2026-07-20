use orbital_resources::MeshDescriptor;

pub trait Shape: Send + Sync {
    fn generate(&self) -> MeshDescriptor;
    fn name(&self) -> &str;
}

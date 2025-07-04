/// Defines the type of "thing" to import from a glTF file.
#[derive(Debug)]
pub enum GltfImportType {
    Scene,
    Model,
    Camera,
    // TODO: Light,
    // TODO: Animation,
}

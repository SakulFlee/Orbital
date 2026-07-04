mod model;
pub use model::*;

mod camera;
pub use camera::*;

mod environment;
pub use environment::*;

mod light;
pub use light::*;

use orbital_importer_gltf::ImportTask;

#[derive(Debug)]
pub enum WorldEvent {
    Model(ModelEvent),
    Camera(CameraEvent),
    Environment(EnvironmentEvent),
    Light(LightEvent),
    Import(ImportTask),
    Clear,
}

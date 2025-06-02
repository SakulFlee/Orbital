mod store;
pub use store::*;

// TODO: Frustum Checking should be a separate System (outside of Renderer)
// TODO: Result of Frustum Check + Active Camera should be extracted from World and be provided when **calling** the RenderSystem
// TODO: Frustum Check -is-> System --> Rename to "ObserverSystem"
// TODO: Renderer -is-> System
// TODO: Physics -is-> System
// TODO: World should handle WorldEvents (previously PhysicsEvents)
// TODO: AFTER lighting and rendering has been fully implemented, figure out how to implement a physics system (probably 3rd-party).

pub struct World {
    model_store: ModelStore,
    camera_store: CameraStore,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    pub fn new() -> Self {
        Self {
            model_store: ModelStore::new(),
            camera_store: CameraStore::new(),
        }
    }

    pub fn model_store(&mut self) -> &mut ModelStore {
        &mut self.model_store
    }

    pub fn camera_store(&mut self) -> &mut CameraStore {
        &mut self.camera_store
    }
}

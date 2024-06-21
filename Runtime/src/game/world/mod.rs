use wgpu::{Device, Queue};

use crate::{
    log::{error, info},
    resources::realizations::{Composition, Model},
};

pub mod change;
pub use change::*;

#[derive(Default)]
pub struct World {
    pub composition: Composition,
    pub changes_queue: Vec<WorldChangeDescriptor>,
}

impl World {
    /// This adds a [WorldChangeDescriptor] to the internal queue.
    /// In time, these [WorldChangeDescriptor]s will be processed and realized.
    /// If, for whatever reason, the realization process fails, an error will be
    /// printed to the application log. This does NOT panic the process, but
    /// things might be missing in your [World].
    pub fn queue_world_change(&mut self, world_change: WorldChangeDescriptor) {
        self.changes_queue.push(world_change);
    }

    pub fn update(&mut self, device: &Device, queue: &Queue) {
        // Pull all changes out of the queue, then process
        let drain = self.changes_queue.drain(..);

        for change in drain {
            match change {
                WorldChangeDescriptor::SwitchComposition(descriptor) => {
                    match Composition::from_descriptor(&descriptor, device, queue) {
                        Ok(composition) => {
                            self.composition = composition;

                            info!(
                                "Switching composition successful! Models present: {} (empty? {})",
                                self.composition.size(),
                                self.composition.is_empty()
                            );
                        }
                        Err(e) => error!("Failed switching to a new composition: {:#?}", e),
                    }
                }
                WorldChangeDescriptor::SpawnModels(model_descriptors) => {
                    for model_descriptor in model_descriptors {
                        match Model::from_descriptor(&model_descriptor, device, queue) {
                            Ok(model) => self.composition.add_model(model),
                            Err(e) => error!("Failed adding a model to composition: {:#?}", e),
                        }
                    }
                }
            }
        }
    }
}

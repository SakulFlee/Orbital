use std::sync::Arc;

use wgpu::{CommandBuffer, CommandEncoderDescriptor};

use crate::engine::Engine;

use self::{object::Object, renderable::Renderable, updateable::Updateable};

pub mod clear_screen_object;
pub mod object;
pub mod renderable;
pub mod updateable;

pub struct AppWorld {
    objects: Vec<Box<dyn Object>>,
    only_updateable: Vec<Box<dyn Updateable>>,
    only_renderable: Vec<Box<dyn Renderable>>,
}

impl AppWorld {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            only_updateable: Vec::new(),
            only_renderable: Vec::new(),
        }
    }

    pub fn spawn_object(&mut self, object: Box<dyn Object>) {
        self.objects.push(object);
    }

    pub fn spawn_updateable(&mut self, updateable: Box<dyn Updateable>) {
        self.only_updateable.push(updateable);
    }

    pub fn spawn_renderable(&mut self, renderable: Box<dyn Renderable>) {
        self.only_renderable.push(renderable);
    }

    pub fn call_updateables(&mut self) {
        self.only_updateable
            .iter_mut()
            .for_each(|updateable| updateable.update());
        self.objects
            .iter_mut()
            .for_each(|updateable| updateable.update());
    }

    pub fn call_renderables(&mut self, engine: Arc<Engine>) -> Vec<CommandBuffer> {
        // Index for [`CommandEncoder`] label
        let mut index = 0;

        // Process only renderable objects
        let command_buffers_0: Vec<CommandBuffer> = self
            .only_renderable
            .iter_mut()
            .filter(|x| x.do_render())
            .map(|x| {
                // Create new [`CommandEncoder`]
                let command_encoder =
                    engine
                        .get_device()
                        .create_command_encoder(&CommandEncoderDescriptor {
                            label: Some(&format!("REnc#{index}")),
                        });
                // Increment index after being used
                index += 1;

                // Call render function
                x.render(engine.clone(), command_encoder)
            })
            .collect();

        // Process full objects
        let command_buffers_1: Vec<CommandBuffer> = self
            .objects
            .iter_mut()
            .filter(|x| x.do_render())
            .map(|x| {
                // Create new [`CommandEncoder`]
                let command_encoder =
                    engine
                        .get_device()
                        .create_command_encoder(&CommandEncoderDescriptor {
                            label: Some(&format!("REnc#{index}")),
                        });
                // Increment index after being used
                index += 1;

                // Call render function
                x.render(engine.clone(), command_encoder)
            })
            .collect();

        let mut command_buffers: Vec<CommandBuffer> = Vec::new();
        command_buffers.extend(command_buffers_0);
        command_buffers.extend(command_buffers_1);
        command_buffers
    }

    pub fn count_object(&self) -> usize {
        self.objects.iter().count()
    }

    pub fn count_updateable(&self) -> usize {
        self.only_updateable.iter().count()
    }

    pub fn count_renderable(&self) -> usize {
        self.only_renderable.iter().count()
    }
}

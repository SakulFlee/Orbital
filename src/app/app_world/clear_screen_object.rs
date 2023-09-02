use std::sync::Arc;

use wgpu::{CommandBuffer, CommandEncoder};

use crate::engine::Engine;

use super::{object::Object, renderable::Renderable, updateable::Updateable};

pub struct ClearScreenObject {
    counter_update: u32,
    counter_render: u32,
}

impl ClearScreenObject {
    pub fn new() -> Self {
        Self {
            counter_update: 0,
            counter_render: 0,
        }
    }
}

impl Object for ClearScreenObject {}

impl Updateable for ClearScreenObject {
    fn update(&mut self) {
        self.counter_update += 1;

        // if self.counter_update % 100 == 0 {
        //     log::debug!("Update: #{}", self.counter_update);
        // }
    }
}

impl Renderable for ClearScreenObject {
    fn render(&mut self, _engine: Arc<Engine>, _command_encoder: CommandEncoder) -> CommandBuffer {
        self.counter_render += 1;

        // if self.counter_render % 100 == 0 {
        //     log::debug!("Render: #{}", self.counter_render);
        // }

        // TODO: Move actual clear screen logic into here.

        todo!()
    }

    fn do_render(&self) -> bool {
        false // -> Render once, then use the buffer
    }
}

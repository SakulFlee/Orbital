use std::sync::Arc;

use wgpu::{CommandBuffer, CommandEncoder};

use crate::engine::Engine;

pub trait Renderable {
    fn render(&mut self, engine: Arc<Engine>, command_encoder: CommandEncoder) -> CommandBuffer;

    fn do_render(&self) -> bool;
}

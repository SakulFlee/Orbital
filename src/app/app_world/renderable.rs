use wgpu::{CommandBuffer, CommandEncoder, TextureView};

pub trait Renderable {
    fn render(
        &mut self,
        command_encoder: CommandEncoder,
        texture_view: &TextureView,
    ) -> CommandBuffer;

    fn do_render(&self) -> bool;
}

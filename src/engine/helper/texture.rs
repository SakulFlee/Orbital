use wgpu::{Texture, TextureView, TextureViewDescriptor};

pub trait TextureHelper {
    fn make_texture_view(&self) -> TextureView;
    fn make_texture_view_descriptor(&self, descriptor: &TextureViewDescriptor) -> TextureView;
}

impl TextureHelper for Texture {
    fn make_texture_view(&self) -> TextureView {
        self.make_texture_view_descriptor(&TextureViewDescriptor::default())
    }

    fn make_texture_view_descriptor(&self, descriptor: &TextureViewDescriptor) -> TextureView {
        self.create_view(descriptor)
    }
}

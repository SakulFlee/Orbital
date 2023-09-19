use wgpu::{SurfaceTexture, TextureView, TextureViewDescriptor};

pub trait SurfaceTextureHelper {
    fn make_texture_view(&self) -> TextureView;
    fn make_texture_view_descriptor(&self, descriptor: &TextureViewDescriptor) -> TextureView;
}

impl SurfaceTextureHelper for SurfaceTexture {
    fn make_texture_view(&self) -> TextureView {
        self.make_texture_view_descriptor(&TextureViewDescriptor::default())
    }

    fn make_texture_view_descriptor(&self, descriptor: &TextureViewDescriptor) -> TextureView {
        self.texture.create_view(descriptor)
    }
}

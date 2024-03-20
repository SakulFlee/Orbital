use texture_trait::TextureTrait;
use wgpu::{Sampler, Texture, TextureView};

pub struct TextureWGPU {
    texture: Texture,
    texture_view: TextureView,
    sampler: Sampler,
}

impl TextureTrait<Texture, TextureView, Sampler> for TextureWGPU {
    fn texture(&self) -> &Texture {
        &self.texture
    }

    fn view(&self) -> &TextureView {
        &self.texture_view
    }

    fn sampler(&self) -> &Sampler {
        &self.sampler
    }
}

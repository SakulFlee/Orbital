use wgpu::{Sampler, Texture, TextureView};

pub trait TTexture {
    fn texture(&self) -> &Texture;
    fn view(&self) -> &TextureView;
    fn sampler(&self) -> &Sampler;
}

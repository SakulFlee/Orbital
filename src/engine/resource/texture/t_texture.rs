use wgpu::{Sampler, Texture, TextureView};

pub trait TTexture {
    fn get_texture(&self) -> &Texture;
    fn get_view(&self) -> &TextureView;
    fn get_sampler(&self) -> &Sampler;
}

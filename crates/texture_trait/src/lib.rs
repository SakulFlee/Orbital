pub trait TextureTrait<Texture, TextureView, Sampler> {
    fn texture(&self) -> &Texture;
    fn view(&self) -> &TextureView;
    fn sampler(&self) -> &Sampler;
}

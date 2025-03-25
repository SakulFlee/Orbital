use texture::Texture;
use wgpu::Buffer;

#[derive(Debug)]
pub enum Variable {
    Buffer(Buffer),
    Texture(Texture),
}

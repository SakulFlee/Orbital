use texture::Texture;
use wgpu::Buffer;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Variable {
    Buffer(Buffer),
    Texture(Texture),
}

use wgpu::Buffer;

use orbital_texture::Texture;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Variable {
    Buffer(Buffer),
    Texture(Texture),
}

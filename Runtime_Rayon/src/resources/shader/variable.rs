use wgpu::Buffer;

use crate::resources::Texture;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Variable {
    Buffer(Buffer),
    Texture(Texture),
}

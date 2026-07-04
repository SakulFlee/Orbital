use wgpu::Buffer;

use crate::Texture;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Variable {
    Buffer(Buffer),
    Texture(Texture),
}

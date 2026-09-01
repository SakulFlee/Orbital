use std::num::NonZero;

use wgpu::{BufferBindingType, BufferUsages};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct BufferDescriptor {
    pub data: Vec<u8>,
    pub ty: BufferBindingType,
    pub usage: BufferUsages,
    pub has_dynamic_offset: bool,
    pub min_binding_size: Option<NonZero<u64>>,
    pub count: Option<NonZero<u32>>,
}

impl Default for BufferDescriptor {
    /// Default is an empty Uniform.
    /// Only `data` has to be overwritten, rest can use `..Default::default()`.
    fn default() -> Self {
        Self {
            data: Vec::new(),
            ty: BufferBindingType::Uniform,
            usage: BufferUsages::UNIFORM,
            has_dynamic_offset: false,
            min_binding_size: None,
            count: None,
        }
    }
}

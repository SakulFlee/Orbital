use std::num::NonZero;

use wgpu::{BufferBindingType, BufferUsages};

use crate::BufferDescriptor;

#[test]
fn test_default_buffer_descriptor() {
    let default_descriptor = BufferDescriptor::default();
    assert_eq!(default_descriptor.data.len(), 0);
    assert_eq!(default_descriptor.ty, BufferBindingType::Uniform);
    assert_eq!(default_descriptor.usage, BufferUsages::UNIFORM);
    assert_eq!(default_descriptor.has_dynamic_offset, false);
    assert_eq!(default_descriptor.min_binding_size, None);
    assert_eq!(default_descriptor.count, None);
}

#[test]
fn test_buffer_descriptor_construction() {
    let data = vec![1, 2, 3];
    let descriptor = BufferDescriptor {
        data: data.clone(),
        ty: BufferBindingType::Storage { read_only: true },
        usage: BufferUsages::STORAGE,
        has_dynamic_offset: true,
        min_binding_size: Some(NonZero::new(16).unwrap()),
        count: Some(NonZero::new(2).unwrap()),
    };

    assert_eq!(descriptor.data, data);
    assert_eq!(
        descriptor.ty,
        BufferBindingType::Storage { read_only: true }
    );
    assert_eq!(descriptor.usage, BufferUsages::STORAGE);
    assert_eq!(descriptor.has_dynamic_offset, true);
    assert_eq!(descriptor.min_binding_size, Some(NonZero::new(16).unwrap()));
    assert_eq!(descriptor.count, Some(NonZero::new(2).unwrap()));
}

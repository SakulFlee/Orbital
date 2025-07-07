use rand::{rng, Rng};
use wgpu::{SamplerBindingType, TextureSampleType};

use crate::{
    resources::{
        BufferDescriptor, ShaderDescriptor, ShaderSource, TextureDescriptor, VariableType,
    },
    wgpu_test_adapter,
};

#[derive(Debug)]
struct TestImplementation {
    variables: Vec<VariableType>,
    buffer_count: u32,
    texture_count: u32,
}

impl TestImplementation {
    fn do_work(&mut self) {
        if self.buffer_count > 0 || self.texture_count > 0 {
            self.variables = Vec::new();

            for _ in 0..self.buffer_count {
                self.variables.push(VariableType::Buffer(BufferDescriptor {
                    data: vec![0u8],
                    ..Default::default()
                }));
            }

            for _ in 0..self.texture_count {
                self.variables.push(VariableType::Texture {
                    descriptor: TextureDescriptor::uniform_luma_black(),
                    sample_type: TextureSampleType::Float { filterable: false },
                    sampler_binding_type: SamplerBindingType::Filtering,
                });
            }
        }
    }
}

impl ShaderDescriptor for TestImplementation {
    fn source(&self) -> ShaderSource {
        ShaderSource::String("")
    }

    fn variables(&self) -> Option<&Vec<VariableType>> {
        (!self.variables.is_empty()).then_some(&self.variables)
    }
}

fn test(buffer_count: u32, texture_count: u32) {
    let (_, device, queue) = wgpu_test_adapter::make_wgpu_connection();

    let mut test_impl = TestImplementation {
        variables: Vec::new(),
        buffer_count,
        texture_count,
    };
    test_impl.do_work();
    println!("{test_impl:?}");

    let (_bind_group, _bind_group_layout, variables) = test_impl
        .bind_group(&device, &queue)
        .expect("Acquiring BindGroup failed");

    for (k, v) in &*variables {
        println!("# {k}: {v:?}");
    }

    let total_indices_expected = buffer_count as usize + texture_count as usize;
    assert_eq!(total_indices_expected, variables.len());
}

/// Attempts to create an empty bind group.
#[test]
fn test_empty() {
    test(0, 0);
}

/// Attempts to create a bind group with a random amount of buffer variables.
#[test]
fn test_buffer_count_random() {
    let mut rng = rng();
    let count = rng.random_range(1..=12);
    test(count, 0);
}

/// Attempts to create a bind group with a random amount of texture variables.
#[test]
fn test_texture_count_random() {
    let mut rng = rng();
    let count = rng.random_range(1..=12);
    test(0, count);
}

/// Attempts to create a bind group with a random amount of texture variables.
#[test]
fn test_buffer_and_texture_count_random() {
    let mut rng = rng();
    let buffer_count = rng.random_range(1..=12);
    let texture_count = rng.random_range(1..=12);
    test(buffer_count, texture_count);
}

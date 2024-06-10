use std::fmt::Write;
use wgpu::{Device, Queue, ShaderModule, ShaderModuleDescriptor, ShaderSource};

use crate::{error::Error, resources::ShaderDescriptor};

#[derive(Debug)]
pub struct Shader {
    shader_module: ShaderModule,
}

impl Shader {
    pub fn from_descriptor(
        shader_descriptor: ShaderDescriptor,
        device: &Device,
        _queue: &Queue,
    ) -> Result<Self, Error> {
        let shader_module = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(shader_descriptor.into()),
        });

        Ok(Self { shader_module })
    }

    pub fn compile_shader(source: &str, includes: &str) -> String {
        let last_preprocessor_line = source
            .lines()
            .enumerate()
            .filter(|x| x.1.starts_with('#'))
            .map(|x| x.0)
            .max()
            .expect("Shader doesn't seem to be annotated by preprocessor! (Missing #version xyz?)");

        let preprocessor_lines =
            source
                .lines()
                .take(last_preprocessor_line + 1)
                .fold(String::new(), |mut output, x| {
                    let _ = writeln!(output, "{x}");
                    output
                });
        let rest_of_shader =
            source
                .lines()
                .skip(last_preprocessor_line + 1)
                .fold(String::new(), |mut output, x| {
                    let _ = writeln!(output, "{x}");
                    output
                });

        format!("{preprocessor_lines}\n{includes}\n{rest_of_shader}")
    }

    pub fn from_existing(shader_module: ShaderModule) -> Self {
        Self { shader_module }
    }

    pub fn shader_module(&self) -> &ShaderModule {
        &self.shader_module
    }
}

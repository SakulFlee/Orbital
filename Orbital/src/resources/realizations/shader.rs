use std::{fmt::Write, sync::Arc};
use wgpu::{Device, Queue, ShaderModule, ShaderModuleDescriptor, ShaderSource};

use crate::{
    error::Error,
    resources::descriptors::{self, ShaderDescriptor, ShaderResource},
};

#[derive(Debug)]
pub struct Shader {
    module: ShaderModule,
    resources: Vec<ShaderResource>,
}

impl Shader {
    pub fn from_descriptor(
        descriptor: Arc<ShaderDescriptor>,
        device: &Device,
        _queue: &Queue,
    ) -> Result<Self, Error> {
        let shader_module = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl((*descriptor.source).into()),
        });

        Ok(Self {
            module: shader_module,
            resources: descriptor.resource_groups,
        })
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
        Self {
            module: shader_module,
        }
    }

    pub fn shader_module(&self) -> &ShaderModule {
        &self.module
    }
}

use std::hash::{DefaultHasher, Hash, Hasher};

use wgpu::{
    naga::{FastHashMap, ShaderStage},
    Device, Queue, ShaderModule, ShaderModuleDescriptor, ShaderSource,
};

use crate::{error::Error, resources::ShaderDescriptor};

pub struct Shader {
    vertex_shader_module: ShaderModule,
    fragment_shader_module: ShaderModule,
}

impl Shader {
    pub fn from_descriptor(
        shader_descriptor: &ShaderDescriptor,
        device: &Device,
        queue: &Queue,
    ) -> Result<Self, Error> {
        let includes_shader_source = shader_descriptor
            .includes
            .iter()
            .map(|s| format!("{s}\n"))
            .collect::<String>();

        let vertex_shader_source = format!(
            "{}\n{}",
            includes_shader_source.clone(),
            shader_descriptor.vertex_source
        );
        let fragment_shader_source = format!(
            "{}\n{}",
            includes_shader_source, shader_descriptor.fragment_source
        );

        let vertex_shader_module = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Glsl {
                shader: vertex_shader_source.into(),
                stage: ShaderStage::Vertex,
                defines: FastHashMap::default(),
            },
        });
        let fragment_shader_module = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Glsl {
                shader: fragment_shader_source.into(),
                stage: ShaderStage::Fragment,
                defines: FastHashMap::default(),
            },
        });

        Ok(Self {
            vertex_shader_module,
            fragment_shader_module,
        })
    }

    pub fn from_existing(
        vertex_shader_module: ShaderModule,
        fragment_shader_module: ShaderModule,
    ) -> Self {
        Self {
            vertex_shader_module,
            fragment_shader_module,
        }
    }

    pub fn vertex_shader_module(&self) -> &ShaderModule {
        &self.vertex_shader_module
    }

    pub fn fragment_shader_module(&self) -> &ShaderModule {
        &self.fragment_shader_module
    }
}

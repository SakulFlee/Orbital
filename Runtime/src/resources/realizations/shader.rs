


use wgpu::{
    Device, Queue, ShaderModule, ShaderModuleDescriptor, ShaderSource,
};

use crate::{error::Error, resources::ShaderDescriptor};

pub struct Shader {
    shader_module: ShaderModule,
    // vertex_shader_module: ShaderModule,
    // fragment_shader_module: ShaderModule,
}

impl Shader {
    pub fn from_descriptor(
        shader_descriptor: ShaderDescriptor,
        device: &Device,
        _queue: &Queue,
    ) -> Result<Self, Error> {
        // let includes_shader_source = shader_descriptor
        //     .includes
        //     .iter()
        //     .map(|s| format!("{s}\n"))
        //     .collect::<String>();

        // let vertex_shader_source =
        //     Self::compile_shader(&shader_descriptor.vertex_source, &includes_shader_source);
        // debug!("Compiled VERTEX Shader:\n{vertex_shader_source}");

        // let fragment_shader_source =
        //     Self::compile_shader(&shader_descriptor.fragment_source, &includes_shader_source);
        // debug!("Compiled FRAGMENT Shader:\n{fragment_shader_source}");

        // let vertex_shader_module = device.create_shader_module(ShaderModuleDescriptor {
        //     label: None,
        //     source: ShaderSource::Glsl {
        //         shader: vertex_shader_source.into(),
        //         stage: ShaderStage::Vertex,
        //         defines: FastHashMap::default(),
        //     },
        // });
        // let fragment_shader_module = device.create_shader_module(ShaderModuleDescriptor {
        //     label: None,
        //     source: ShaderSource::Glsl {
        //         shader: fragment_shader_source.into(),
        //         stage: ShaderStage::Fragment,
        //         defines: FastHashMap::default(),
        //     },
        // });

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

        let preprocessor_lines = source
            .lines()
            .take(last_preprocessor_line + 1)
            .map(|x| format!("{x}\n"))
            .collect::<String>();
        let rest_of_shader = source
            .lines()
            .skip(last_preprocessor_line + 1)
            .map(|x| format!("{x}\n"))
            .collect::<String>();

        format!("{preprocessor_lines}\n{includes}\n{rest_of_shader}")
    }

    pub fn from_existing(
        shader_module: ShaderModule,
        // vertex_shader_module: ShaderModule,
        // fragment_shader_module: ShaderModule,
    ) -> Self {
        Self {
            shader_module,
            // vertex_shader_module,
            // fragment_shader_module,
        }
    }

    pub fn shader_module(&self) -> &ShaderModule {
        &self.shader_module
    }

    // pub fn vertex_shader_module(&self) -> &ShaderModule {
    //     &self.vertex_shader_module
    // }

    // pub fn fragment_shader_module(&self) -> &ShaderModule {
    //     &self.fragment_shader_module
    // }
}

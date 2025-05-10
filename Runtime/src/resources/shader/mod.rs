use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BindingResource, BindingType, Device, Queue, SamplerBindingType, ShaderModule,
    ShaderModuleDescriptor, ShaderStages,
};

use shader_preprocessor::ShaderPreprocessor;
use texture::Texture;

mod error;
pub use error::*;

mod variable;
pub use variable::*;

mod variables;
pub use variables::*;

mod variable_type;
pub use variable_type::*;

mod source;
pub use source::*;

#[cfg(test)]
mod tests;

pub trait ShaderDescriptor {
    fn name(&self) -> Option<String> {
        None
    }

    fn source(&self) -> ShaderSource;

    fn variables(&self) -> Option<Vec<VariableType>> {
        None
    }

    fn stages(&self) -> ShaderStages {
        ShaderStages::all()
    }

    fn shader_preprocessor(&self) -> Option<ShaderPreprocessor> {
        None
    }

    fn shader_module(&self, device: &Device) -> Result<ShaderModule, ShaderError> {
        // TODO: Need a cache here
        let preprocessor = self.shader_preprocessor().unwrap_or(
            ShaderPreprocessor::new_with_defaults()
                .map_err(|e| ShaderError::ShaderPreprocessor(e))?,
        );

        let shader_source = self.source().read_as_string()?;

        let preprocessed_source = preprocessor
            .parse_shader(shader_source)
            .map_err(|e| ShaderError::ShaderPreprocessor(e))?;

        Ok(device.create_shader_module(ShaderModuleDescriptor {
            label: self.name().as_deref(),
            source: wgpu::ShaderSource::Wgsl(preprocessed_source.into()),
        }))
    }

    fn bind_group_layout(
        &self,
        device: &Device,
        queue: &Queue,
    ) -> Result<(BindGroupLayout, Variables), ShaderError> {
        let mut entries = Vec::new();
        let mut variables: Variables = Variables::new();

        let mut binding_count = 0;
        if let Some(variable_types) = self.variables() {
            for variable_type in variable_types {
                match variable_type {
                    VariableType::Buffer(buffer_descriptor) => {
                        let buffer = device.create_buffer_init(&BufferInitDescriptor {
                            label: None,
                            usage: buffer_descriptor.usage,
                            contents: &buffer_descriptor.data,
                        });
                        variables.insert(binding_count, Variable::Buffer(buffer));

                        let entry = BindGroupLayoutEntry {
                            binding: binding_count,
                            visibility: self.stages(),
                            ty: BindingType::Buffer {
                                ty: buffer_descriptor.ty,
                                has_dynamic_offset: buffer_descriptor.has_dynamic_offset,
                                min_binding_size: buffer_descriptor.min_binding_size,
                            },
                            count: buffer_descriptor.count,
                        };
                        entries.push(entry);

                        binding_count += 1;
                    }
                    VariableType::Texture {
                        descriptor,
                        sampler_type: sample_type,
                    } => {
                        // Note:
                        // We are skipping over the sampler binding as it is already contained inside the `Texture` realization!
                        // WGPU handles them as two separate resources (thus two binding indices), but we are treating it as **one**.
                        // Regardless, we still need to skip over the binding index of the sampler, as later we will do the same in reverse: 1x `Texture` == 1x Texture binding + 1x Sampler binding.

                        let insert_index = binding_count;
                        let texture = Texture::from_descriptor(&descriptor, device, queue)
                            .map_err(|e| ShaderError::Texture(e))?;

                        let texture_binding = BindGroupLayoutEntry {
                            binding: binding_count,
                            visibility: self.stages(),
                            ty: BindingType::Texture {
                                sample_type: sample_type,
                                view_dimension: *texture.view_dimension(),
                                multisampled: false,
                            },
                            count: None,
                        };
                        entries.push(texture_binding);
                        binding_count += 1;

                        let sampler_binding = BindGroupLayoutEntry {
                            binding: binding_count,
                            visibility: self.stages(),
                            ty: BindingType::Sampler(SamplerBindingType::Filtering),
                            count: None,
                        };
                        entries.push(sampler_binding);
                        binding_count += 1;

                        variables.insert(insert_index, Variable::Texture(texture));
                    }
                }
            }
        }

        Ok((
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: self.name().as_deref(),
                entries: &entries,
            }),
            variables,
        ))
    }

    fn bind_group(
        &self,
        device: &Device,
        queue: &Queue,
    ) -> Result<(BindGroup, BindGroupLayout, Variables), ShaderError> {
        let (layout, variables) = self.bind_group_layout(device, queue)?;

        let mut binds = Vec::new();
        let mut binding_index = 0u32;

        if let Some(variable_types) = self.variables() {
            for var in variable_types {
                match var {
                    VariableType::Buffer(_) => {
                        let buffer = if let Variable::Buffer(buffer) = variables
                            .get(&binding_index)
                            .expect("Expected Variable to exist!")
                        {
                            buffer
                        } else {
                            panic!("Expected Buffer but got unexpected type!");
                        };

                        binds.push(BindGroupEntry {
                            binding: binding_index,
                            resource: BindingResource::Buffer(buffer.as_entire_buffer_binding()),
                        });
                        binding_index += 1;
                    }
                    VariableType::Texture {
                        descriptor: _,
                        sampler_type: _,
                    } => {
                        // Note:
                        // Check `bind_group_layout` above for information on why the binding index is skipped here.

                        let texture = if let Variable::Texture(texture) = variables
                            .get(&binding_index)
                            .expect("Expected Variable to exist!")
                        {
                            texture
                        } else {
                            panic!("Expected Texture but got unexpected type!");
                        };

                        binds.push(BindGroupEntry {
                            binding: binding_index,
                            resource: BindingResource::TextureView(texture.view()),
                        });
                        binding_index += 1;

                        binds.push(BindGroupEntry {
                            binding: binding_index,
                            resource: BindingResource::Sampler(texture.sampler()),
                        });
                        binding_index += 1;
                    }
                }
            }
        }

        Ok((
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: self.name().as_deref(),
                layout: &layout,
                entries: &binds,
            }),
            layout,
            variables,
        ))
    }
}

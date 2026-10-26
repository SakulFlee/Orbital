use std::sync::Arc;

use wgpu::{
    BindGroup, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BindingResource, BindingType, Device, Queue, ShaderModule, ShaderModuleDescriptor,
    ShaderStages,
    util::{BufferInitDescriptor, DeviceExt},
};

use orbital_shader_preprocessor::{NodeRegistry, ShaderBuilder};
use orbital_texture::Texture;

mod error;
pub use error::*;

mod stats;
pub use stats::*;

mod variable;
pub use variable::*;

mod variables;
pub use variables::*;

mod variable_type;
pub use variable_type::*;

/// A shader is assembled from named [`orbital_shader_preprocessor::ShaderNode`]s
/// resolved against a [`NodeRegistry`], optionally followed by raw WGSL
/// (e.g. an entrypoint body). See [`ShaderDescriptor`].
pub trait ShaderDescriptor {
    fn name(&self) -> Option<String> {
        None
    }

    /// The root node names (resolved against the registry) that make up this
    /// shader. Dependencies of each node are resolved transitively.
    fn nodes(&self) -> &[&str];

    /// Optional raw WGSL appended after the resolved nodes (e.g. entrypoints).
    fn raw_source(&self) -> Option<Arc<str>> {
        None
    }

    /// The registry to resolve nodes against. Defaults to the process-wide
    /// global registry, which is preloaded with the engine prelude.
    fn registry(&self) -> Arc<NodeRegistry> {
        Arc::new(NodeRegistry::global().clone())
    }

    fn variables(&self) -> Option<&Vec<VariableType>> {
        None
    }

    fn stages(&self) -> ShaderStages {
        ShaderStages::all()
    }

    fn shader_module(&self, device: &Device) -> Result<ShaderModule, ShaderError> {
        let mut builder = ShaderBuilder::new(self.registry());
        builder
            .add_nodes(self.nodes())
            .map_err(ShaderError::ShaderPreprocessor)?;

        if let Some(raw) = self.raw_source() {
            builder.add_source(raw);
        }

        let source = builder.build();
        stats::record_source(&source);

        Ok(device.create_shader_module(ShaderModuleDescriptor {
            label: self.name().as_deref(),
            source: wgpu::ShaderSource::Wgsl(source.into()),
        }))
    }

    /// Bind-group layout entries derived from the variable types.
    ///
    /// This does not realize textures or buffers, so a shader program can own
    /// its layout before material data exists.
    fn bind_group_layout_entries(&self) -> Vec<BindGroupLayoutEntry> {
        let Some(variable_types) = self.variables() else {
            return Vec::new();
        };

        let mut entries = Vec::new();
        let mut binding = 0u32;

        for variable_type in variable_types {
            match variable_type {
                VariableType::Buffer(buffer_descriptor) => {
                    entries.push(BindGroupLayoutEntry {
                        binding,
                        visibility: self.stages(),
                        ty: BindingType::Buffer {
                            ty: buffer_descriptor.ty,
                            has_dynamic_offset: buffer_descriptor.has_dynamic_offset,
                            min_binding_size: buffer_descriptor.min_binding_size,
                        },
                        count: buffer_descriptor.count,
                    });
                    binding += 1;
                }
                VariableType::Texture {
                    descriptor,
                    sample_type,
                    sampler_binding_type,
                } => {
                    // A texture variable occupies a texture binding plus a
                    // sampler binding. `Variables` stores the texture under the
                    // first index and the sampler is read from the texture.
                    entries.push(BindGroupLayoutEntry {
                        binding,
                        visibility: self.stages(),
                        ty: BindingType::Texture {
                            sample_type: *sample_type,
                            view_dimension: descriptor.view_dimension(),
                            multisampled: false,
                        },
                        count: None,
                    });
                    binding += 1;

                    entries.push(BindGroupLayoutEntry {
                        binding,
                        visibility: self.stages(),
                        ty: BindingType::Sampler(*sampler_binding_type),
                        count: None,
                    });
                    binding += 1;
                }
            }
        }

        entries
    }

    /// Create only the bind-group layout, without realizing material data.
    fn bind_group_layout_only(&self, device: &Device) -> Option<BindGroupLayout> {
        let entries = self.bind_group_layout_entries();
        if entries.is_empty() {
            return None;
        }

        Some(device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: self.name().as_deref(),
            entries: &entries,
        }))
    }

    /// Realize texture/buffer variables without creating a bind group.
    fn realize_variables(
        &self,
        device: &Device,
        queue: &Queue,
    ) -> Result<Option<Variables>, ShaderError> {
        let Some(variable_types) = self.variables() else {
            return Ok(None);
        };
        if variable_types.is_empty() {
            return Ok(None);
        }

        let mut variables = Variables::new();
        let mut binding = 0u32;

        for variable_type in variable_types {
            match variable_type {
                VariableType::Buffer(buffer_descriptor) => {
                    let buffer = device.create_buffer_init(&BufferInitDescriptor {
                        label: None,
                        usage: buffer_descriptor.usage,
                        contents: &buffer_descriptor.data,
                    });
                    variables.insert(binding, Variable::Buffer(buffer));
                    binding += 1;
                }
                VariableType::Texture { descriptor, .. } => {
                    let texture = Texture::from_descriptor(descriptor, device, queue)
                        .map_err(ShaderError::Texture)?;
                    variables.insert(binding, Variable::Texture(texture));
                    binding += 2;
                }
            }
        }

        Ok(Some(variables))
    }

    /// Create a bind group from an already-created layout and variables.
    fn bind_group_with_layout(
        &self,
        layout: &BindGroupLayout,
        variables: &Variables,
        device: &Device,
    ) -> Result<Option<BindGroup>, ShaderError> {
        let Some(variable_types) = self.variables() else {
            return Ok(None);
        };
        if variable_types.is_empty() {
            return Ok(None);
        }

        let mut binds = Vec::new();
        let mut binding = 0u32;

        for variable_type in variable_types {
            match variable_type {
                VariableType::Buffer(_) => {
                    let variable = variables
                        .get(&binding)
                        .ok_or(ShaderError::MissingVariable { binding })?;
                    let buffer = if let Variable::Buffer(buffer) = variable {
                        buffer
                    } else {
                        return Err(ShaderError::VariableTypeMismatch {
                            binding,
                            expected: "Buffer",
                        });
                    };

                    binds.push(BindGroupEntry {
                        binding,
                        resource: BindingResource::Buffer(buffer.as_entire_buffer_binding()),
                    });
                    binding += 1;
                }
                VariableType::Texture { .. } => {
                    let variable = variables
                        .get(&binding)
                        .ok_or(ShaderError::MissingVariable { binding })?;
                    let texture = if let Variable::Texture(texture) = variable {
                        texture
                    } else {
                        return Err(ShaderError::VariableTypeMismatch {
                            binding,
                            expected: "Texture",
                        });
                    };

                    binds.push(BindGroupEntry {
                        binding,
                        resource: BindingResource::TextureView(texture.view()),
                    });
                    binding += 1;

                    binds.push(BindGroupEntry {
                        binding,
                        resource: BindingResource::Sampler(texture.sampler()),
                    });
                    binding += 1;
                }
            }
        }

        Ok(Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: self.name().as_deref(),
            layout,
            entries: &binds,
        })))
    }

    fn bind_group_layout(
        &self,
        device: &Device,
        queue: &Queue,
    ) -> Result<Option<(BindGroupLayout, Variables)>, ShaderError> {
        let Some(layout) = self.bind_group_layout_only(device) else {
            return Ok(None);
        };
        let Some(variables) = self.realize_variables(device, queue)? else {
            return Ok(None);
        };

        Ok(Some((layout, variables)))
    }

    fn bind_group(
        &self,
        device: &Device,
        queue: &Queue,
    ) -> Result<Option<(BindGroup, BindGroupLayout, Variables)>, ShaderError> {
        let Some((layout, variables)) = self.bind_group_layout(device, queue)? else {
            return Ok(None);
        };
        let Some(bind_group) = self.bind_group_with_layout(&layout, &variables, device)? else {
            return Ok(None);
        };

        Ok(Some((bind_group, layout, variables)))
    }
}

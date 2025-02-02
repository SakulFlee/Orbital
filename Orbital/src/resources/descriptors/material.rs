use std::{
    hash::{Hash, Hasher},
    mem,
    sync::Arc,
};

use cgmath::Vector3;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BindingResource, BindingType, Color, Device,
    SamplerBindingType, ShaderStages, 
};

use super::{
    BufferDescriptor, SamplingType, ShaderDescriptor, TextureDescriptor, WorldEnvironmentDescriptor
};

#[derive(Debug, Clone, PartialEq)]
pub enum VariableType {
    Buffer(BufferDescriptor),
    Texture(TextureDescriptor),
    // TODO: BindingType::StorageTexture
}

#[derive(Debug, Clone, PartialEq)]
// TODO: maybe rename to "ShaderDescriptor" to allow for "Model { Mesh + Shader }" but also Compute being "Shader"
pub struct MaterialDescriptorTesting {
    label: Option<String>,
    stages: ShaderStages,
    custom_variables: Vec<VariableType>,
    shader_path: String,
}

impl MaterialDescriptorTesting {
    pub fn make_bind_group_layout(&self, device: &Device) -> BindGroupLayout {
        let mut entries = Vec::new();

        let mut binding_count = 0;
        for var in &self.custom_variables {
            match var {
                VariableType::Buffer(buffer_descriptor) => {
                    let entry = BindGroupLayoutEntry {
                        binding: binding_count,
                        visibility: self.stages,
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
                VariableType::Texture(texture) => {
                    // TODO: Realization -> Texture

                    let texture_binding = BindGroupLayoutEntry {
                        binding: binding_count,
                        visibility: self.stages,
                        ty: BindingType::Texture {
                            sample_type: texture.sample_type,
                            view_dimension: texture.view_dimension,
                            multisampled: false,
                        },
                        count: texture.count,
                    };
                    entries.push(texture_binding);
                    binding_count += 1;

                    let sampler_binding = BindGroupLayoutEntry {
                        binding: binding_count,
                        visibility: self.stages,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: texture.count,
                    };
                    entries.push(sampler_binding);
                    binding_count += 1;
                }
            }
        }

        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: self.label.as_ref().map(|x| x.as_str()),
            entries: &entries,
        })
    }

    fn make_bind_group(&self, device: &Device) -> BindGroup {
        let layout = self.make_bind_group_layout(device);
        let mut binds = Vec::new();

        let mut binding_count = 0u32;

        for var in &self.custom_variables {
            match var {
                VariableType::Buffer(buffer_descriptor) => {
                    let buffer = device.create_buffer_init(&BufferInitDescriptor {
                        label: None,
                        usage: buffer_descriptor.usage,
                        contents: &buffer_descriptor.data,
                    });
                    binds.push(BindGroupEntry {
                        binding: binding_count,
                        resource: BindingResource::Buffer(buffer.as_entire_buffer_binding()),
                    });
                    binding_count += 1;
                }
                VariableType::Texture(texture) => {
                    let texture_view = device.create_texture(&wgpu::TextureDescriptor {
                        label: todo!(),
                        size: todo!(),
                        mip_level_count: todo!(),
                        sample_count: todo!(),
                        dimension: todo!(),
                        format: todo!(),
                        usage: todo!(),
                        view_formats: todo!(),
                    });
                    texture_view.create_view(&TextureViewDescriptor)

                    
                    binds.push(BindingResource {
                        binding: binding_count,
                        resource: BindingResource::TextureView(()),
                    });
                    binding_count += 1;
                }
            }
        }

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &layout,
            entries: &binds,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MaterialDescriptor {
    /// Creates a standard PBR (= Physically-Based-Rendering) material.
    PBR {
        normal: Arc<TextureDescriptor>,
        albedo: Arc<TextureDescriptor>,
        albedo_factor: Vector3<f32>,
        metallic: Arc<TextureDescriptor>,
        metallic_factor: f32,
        roughness: Arc<TextureDescriptor>,
        roughness_factor: f32,
        occlusion: Arc<TextureDescriptor>,
        emissive: Arc<TextureDescriptor>,
        custom_shader: Option<ShaderDescriptor>,
    },
    // TODO
    WorldEnvironment(WorldEnvironmentDescriptor),
    Wireframe(Color),
}

impl MaterialDescriptor {
    pub fn default_world_environment() -> MaterialDescriptor {
        MaterialDescriptor::WorldEnvironment(WorldEnvironmentDescriptor::FromFile {
            skybox_type: super::SkyboxType::Specular { lod: 0 },
            cube_face_size: super::WorldEnvironmentDescriptor::DEFAULT_SIZE,
            path: "Assets/HDRs/kloppenheim_02_puresky_4k.hdr",
            sampling_type: SamplingType::ImportanceSampling,
        })
    }
}

impl Hash for MaterialDescriptor {
    fn hash<H: Hasher>(&self, state: &mut H) {
        mem::discriminant(self).hash(state);
    }
}

impl Eq for MaterialDescriptor {}

use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, Device, Queue, SamplerBindingType,
    ShaderStages, TextureSampleType, TextureViewDimension,
};

use crate::resources::{
    MaterialDescriptor, PipelineDescriptor, ShaderDescriptor, TextureDescriptor,
};

use super::Texture;

pub struct Material {
    bind_group: BindGroup,
    bind_group_layout: BindGroupLayout,
    pipeline_descriptor: PipelineDescriptor,
}

impl Material {
    pub fn from_descriptor(
        descriptor: &MaterialDescriptor,
        device: &Device,
        queue: &Queue,
    ) -> Self {
        match descriptor {
            MaterialDescriptor::PBR(albedo) => Self::standard_pbr(albedo, None, device, queue),
            MaterialDescriptor::PBRCustomShader(albedo, shader_descriptor) => todo!(),
            MaterialDescriptor::Custom(
                bind_group_descriptor,
                bind_group_layout_descriptor,
                pipeline_descriptor,
            ) => Self::from_descriptors(
                bind_group_descriptor,
                bind_group_layout_descriptor,
                pipeline_descriptor.clone(),
                device,
                queue,
            ),
        }
    }

    pub fn standard_pbr(
        albedo_texture_descriptor: &TextureDescriptor,
        shader_descriptor: Option<ShaderDescriptor>,
        device: &Device,
        queue: &Queue,
    ) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Standard PBR"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let albedo_texture = Texture::from_descriptor(albedo_texture_descriptor, device, queue);

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(albedo_texture.view()),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(albedo_texture.sampler()),
                },
            ],
        });

        let pipeline_descriptor = if let Some(shader_descriptor) = shader_descriptor {
            PipelineDescriptor::default_with_shader(shader_descriptor)
        } else {
            PipelineDescriptor::default()
        };

        Self::from_existing(bind_group, bind_group_layout, pipeline_descriptor)
    }

    pub fn from_descriptors(
        bind_group_descriptor: &BindGroupDescriptor,
        bind_group_layout_descriptor: &BindGroupLayoutDescriptor,
        pipeline_descriptor: PipelineDescriptor,
        device: &Device,
        queue: &Queue,
    ) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&bind_group_layout_descriptor);

        let bind_group = device.create_bind_group(&bind_group_descriptor);

        Self::from_existing(bind_group, bind_group_layout, pipeline_descriptor)
    }

    pub fn from_existing(
        bind_group: BindGroup,
        bind_group_layout: BindGroupLayout,
        pipeline_descriptor: PipelineDescriptor,
    ) -> Self {
        Self {
            bind_group,
            bind_group_layout,
            pipeline_descriptor,
        }
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }

    pub fn bind_group_layout(&self) -> &BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn pipeline_descriptor(&self) -> &PipelineDescriptor {
        &self.pipeline_descriptor
    }
}

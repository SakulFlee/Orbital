use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, Device, Queue, TextureFormat,
};

use crate::{
    error::Error,
    resources::{MaterialDescriptor, PipelineDescriptor, ShaderDescriptor, TextureDescriptor},
};

use super::{Pipeline, Texture};

pub struct Material {
    bind_group: BindGroup,
    pipeline_descriptor: PipelineDescriptor,
}

impl Material {
    pub fn from_descriptor(
        descriptor: &MaterialDescriptor,
        surface_format: &TextureFormat,
        device: &Device,
        queue: &Queue,
    ) -> Result<Self, Error> {
        match descriptor {
            MaterialDescriptor::PBR(albedo) => {
                Self::standard_pbr(albedo, None, surface_format, device, queue)
            }
            MaterialDescriptor::PBRCustomShader(albedo, shader_descriptor) => Self::standard_pbr(
                albedo,
                Some(shader_descriptor),
                surface_format,
                device,
                queue,
            ),
            MaterialDescriptor::Custom(bind_group_descriptor, pipeline_descriptor) => Ok(
                Self::from_descriptors(bind_group_descriptor, pipeline_descriptor, device, queue),
            ),
        }
    }

    pub fn standard_pbr(
        albedo_texture_descriptor: &TextureDescriptor,
        shader_descriptor: Option<&ShaderDescriptor>,
        surface_format: &TextureFormat,
        device: &Device,
        queue: &Queue,
    ) -> Result<Self, Error> {
        let albedo_texture = Texture::from_descriptor(albedo_texture_descriptor, device, queue);

        let pipeline_descriptor = if let Some(shader_descriptor) = shader_descriptor {
            PipelineDescriptor::default_with_shader(shader_descriptor)
        } else {
            PipelineDescriptor::default()
        };

        let pipeline =
            Pipeline::from_descriptor(&pipeline_descriptor, surface_format, device, queue)?;

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: pipeline.bind_group_layout(),
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

        Ok(Self::from_existing(bind_group, pipeline_descriptor))
    }

    pub fn from_descriptors(
        bind_group_descriptor: &BindGroupDescriptor,
        pipeline_descriptor: &PipelineDescriptor,
        device: &Device,
        _queue: &Queue,
    ) -> Self {
        let bind_group = device.create_bind_group(bind_group_descriptor);

        Self::from_existing(bind_group, pipeline_descriptor.clone())
    }

    #[cfg(feature = "gltf")]
    pub fn from_gltf(
        gltf_material: &easy_gltf::Material,
        surface_format: &TextureFormat,
        device: &Device,
        queue: &Queue,
    ) -> Result<Self, Error> {
        let pbr = &gltf_material.pbr;

        let albedo_texture_descriptor = if let Some(base_color) = &pbr.base_color_texture {
            TextureDescriptor::StandardSRGBu8Data(base_color.to_vec(), base_color.dimensions())
        } else {
            return Err(Error::NoBaseColor);
        };

        Self::standard_pbr(
            &albedo_texture_descriptor,
            None,
            surface_format,
            device,
            queue,
        )
    }

    pub fn from_existing(bind_group: BindGroup, pipeline_descriptor: PipelineDescriptor) -> Self {
        Self {
            bind_group,
            pipeline_descriptor,
        }
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }

    pub fn pipeline_descriptor(&self) -> &PipelineDescriptor {
        &self.pipeline_descriptor
    }
}

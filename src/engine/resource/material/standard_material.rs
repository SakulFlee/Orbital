use std::path::Path;

use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, SamplerBindingType, ShaderStages,
    TextureSampleType, TextureViewDimension,
};

use crate::engine::{
    DiffuseTexture, EngineResult, LogicalDevice, NormalTexture, ResourceManager, TTexture,
};

use super::TMaterial;

pub struct StandardMaterial {
    name: String,
    diffuse_texture: DiffuseTexture,
    normal_texture: NormalTexture,
    bind_group: BindGroup,
}

impl StandardMaterial {
    pub const BIND_GROUP_LAYOUT_DESCRIPTOR: BindGroupLayoutDescriptor<'static> =
        BindGroupLayoutDescriptor {
            label: Some("Standard Material"),
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
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        };

    pub fn from_path<P>(
        logical_device: &LogicalDevice,
        diffuse_path: P,
        normal_path: P,
    ) -> EngineResult<Self>
    where
        P: AsRef<Path>,
    {
        let diffuse_texture = ResourceManager::diffuse_texture_from_path(
            logical_device,
            diffuse_path.as_ref().clone(),
        )?;

        let normal_texture = ResourceManager::diffuse_texture_from_path(
            logical_device,
            normal_path.as_ref().clone(),
        )?;

        Self::from_texture(logical_device, diffuse_texture, normal_texture)
    }

    pub fn from_texture(
        logical_device: &LogicalDevice,
        diffuse_texture: DiffuseTexture,
        normal_texture: NormalTexture,
    ) -> EngineResult<Self> {
        let bind_group = Self::make_bind_group(
            Some("StandardMaterialBindGroup"),
            &diffuse_texture,
            &normal_texture,
            logical_device,
        );

        Ok(Self {
            name: String::from("StandardMaterial"),
            diffuse_texture,
            normal_texture,
            bind_group,
        })
    }

    fn make_bind_group(
        label: Option<&str>,
        diffuse_texture: &DiffuseTexture,
        normal_texture: &NormalTexture,
        logical_device: &LogicalDevice,
    ) -> BindGroup {
        let bind_group_layout = Self::bind_group_layout(logical_device);

        logical_device
            .device()
            .create_bind_group(&BindGroupDescriptor {
                label,
                layout: &bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(diffuse_texture.view()),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(diffuse_texture.sampler()),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: BindingResource::TextureView(normal_texture.view()),
                    },
                    BindGroupEntry {
                        binding: 3,
                        resource: BindingResource::Sampler(normal_texture.sampler()),
                    },
                ],
            })
    }
}

impl TMaterial for StandardMaterial {
    fn name(&self) -> &str {
        &self.name
    }

    fn diffuse_texture(&self) -> &DiffuseTexture {
        &self.diffuse_texture
    }

    fn normal_texture(&self) -> &NormalTexture {
        &self.normal_texture
    }

    fn bind_group_layout(logical_device: &LogicalDevice) -> BindGroupLayout {
        logical_device
            .device()
            .create_bind_group_layout(&Self::BIND_GROUP_LAYOUT_DESCRIPTOR)
    }

    fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}

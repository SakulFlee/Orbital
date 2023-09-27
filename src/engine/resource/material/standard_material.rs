use std::path::Path;

use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, SamplerBindingType, ShaderStages,
    TextureSampleType, TextureViewDimension,
};

use crate::engine::{DiffuseTexture, EngineResult, LogicalDevice, ResourceManager, TTexture};

use super::TMaterial;

pub struct StandardMaterial {
    name: String,
    diffuse_texture: DiffuseTexture,
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
            ],
        };

    pub fn from_texture<P>(logical_device: &LogicalDevice, file_path: P) -> EngineResult<Self>
    where
        P: AsRef<Path>,
    {
        let file_name = file_path.as_ref().clone().to_str();
        let diffuse_texture =
            ResourceManager::diffuse_texture_from_path(logical_device, file_path.as_ref().clone())?;

        let bind_group = Self::make_bind_group(file_name, &diffuse_texture, logical_device);

        Ok(Self {
            name: format!("StandardMaterial@{}", file_name.unwrap_or("Unknown")),
            diffuse_texture,
            bind_group,
        })
    }

    fn make_bind_group(
        label: Option<&str>,
        diffuse_texture: &DiffuseTexture,
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

    fn bind_group_layout(logical_device: &LogicalDevice) -> BindGroupLayout {
        logical_device
            .device()
            .create_bind_group_layout(&Self::BIND_GROUP_LAYOUT_DESCRIPTOR)
    }

    fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}

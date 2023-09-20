use std::path::Path;

use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, Device, Queue, SamplerBindingType,
    ShaderStages, TextureSampleType, TextureViewDimension,
};

use crate::engine::{DiffuseTexture, EngineResult, ResourceManager, TTexture};

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
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        };

    pub fn from_texture<P>(device: &Device, queue: &Queue, file_path: P) -> EngineResult<Self>
    where
        P: AsRef<Path>,
    {
        let file_name = file_path.as_ref().clone().to_str();
        let diffuse_texture =
            ResourceManager::diffuse_texture_from_path(device, queue, file_path.as_ref().clone())?;

        let bind_group = Self::make_bind_group(file_name, &diffuse_texture, device);

        Ok(Self {
            name: format!("StandardMaterial@{}", file_name.unwrap_or("Unknown")),
            diffuse_texture,
            bind_group,
        })
    }

    fn make_bind_group(
        label: Option<&str>,
        diffuse_texture: &DiffuseTexture,
        device: &Device,
    ) -> BindGroup {
        let bind_group_layout = Self::get_bind_group_layout(device);

        device.create_bind_group(&BindGroupDescriptor {
            label,
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&diffuse_texture.get_view()),
                },
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Sampler(&diffuse_texture.get_sampler()),
                },
            ],
        })
    }
}

impl TMaterial for StandardMaterial {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_diffuse_texture(&self) -> &DiffuseTexture {
        &self.diffuse_texture
    }

    fn get_bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&Self::BIND_GROUP_LAYOUT_DESCRIPTOR)
    }

    fn get_bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}

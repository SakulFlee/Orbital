use cgmath::Vector2;
use wgpu::{
    include_wgsl, AddressMode, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType,
    ComputePassDescriptor, ComputePipeline, ComputePipelineDescriptor, Device, Extent3d,
    FilterMode, PipelineLayoutDescriptor, Queue, SamplerDescriptor, ShaderStages,
    StorageTextureAccess, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    TextureViewDescriptor, TextureViewDimension,
};

use super::Texture;

#[derive(Debug)]
pub struct IblBrdf {
    texture: Option<Texture>,
}

impl IblBrdf {
    const SIZE: u32 = 512;

    pub fn bind_group_layout_descriptor() -> BindGroupLayoutDescriptor<'static> {
        BindGroupLayoutDescriptor {
            label: Some("IBL BRDF"),
            entries: &[
                // Output
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: TextureFormat::Rgba32Float,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        }
    }

    fn create_empty(
        label: Option<&str>,
        size: Vector2<u32>,
        format: TextureFormat,
        usage: TextureUsages,
        device: &Device,
    ) -> Self {
        let texture = device.create_texture(&TextureDescriptor {
            label,
            size: Extent3d {
                width: size.x,
                height: size.y,
                ..Default::default()
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        });

        let view = texture.create_view(&TextureViewDescriptor {
            label,
            dimension: Some(TextureViewDimension::D2),
            ..Default::default()
        });

        let sampler = device.create_sampler(&SamplerDescriptor {
            label,
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            texture: Some(Texture::from_existing(texture, view, sampler)),
        }
    }

    pub fn generate(device: &Device, queue: &Queue) -> Self {
        let dst_texture = Self::create_empty(
            Some("IBL BRDF DST"),
            Vector2 {
                x: Self::SIZE,
                y: Self::SIZE,
            },
            TextureFormat::Rgba32Float,
            TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
            device,
        );

        // Needed only know for processing.
        // Actual rendering later uses the included View in
        // TextureViewDimension::Cube
        let ibl_brdf_output =
            dst_texture
                .texture_ref()
                .texture()
                .create_view(&TextureViewDescriptor {
                    label: Some("IRL BRDF DST view"),
                    dimension: Some(TextureViewDimension::D2),
                    ..Default::default()
                });

        let bind_group_layout =
            device.create_bind_group_layout(&Self::bind_group_layout_descriptor());
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Equirectangular"),
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&ibl_brdf_output),
            }],
        });

        let pipeline = Self::make_pipeline(&bind_group_layout, device);

        let mut encoder = device.create_command_encoder(&Default::default());
        let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("IBL BRDF"),
            ..Default::default()
        });

        let workgroups = (Self::SIZE + 15) / 16;
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(workgroups, workgroups, 1);

        drop(pass);
        queue.submit([encoder.finish()]);

        dst_texture
    }

    fn make_pipeline(bind_group_layout: &BindGroupLayout, device: &Device) -> ComputePipeline {
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[bind_group_layout],
            push_constant_ranges: &[],
        });

        let shader = device.create_shader_module(include_wgsl!("ibl_brdf.wgsl"));

        device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("IRL BRDF"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        })
    }

    pub fn texture_ref(&self) -> &Texture {
        self.texture
            .as_ref()
            .expect("IBL BRDF texture was already taken!")
    }

    pub fn texture(mut self) -> Texture {
        self.texture
            .take()
            .expect("IBL BRDF texture was already taken!")
    }
}

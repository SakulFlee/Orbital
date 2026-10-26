//! Fullscreen HDR → LDR tonemap pass.

use wgpu::{
    BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
    BlendState, BufferBindingType, ColorTargetState, ColorWrites, Device, FragmentState,
    PipelineLayoutDescriptor, PrimitiveState, RenderPassColorAttachment, RenderPassDescriptor,
    RenderPipeline, RenderPipelineDescriptor, SamplerBindingType, SamplerDescriptor,
    ShaderModuleDescriptor, ShaderSource, ShaderStages, StoreOp, TextureSampleType, TextureView,
    TextureViewDimension, VertexState,
    util::{BufferInitDescriptor, DeviceExt},
};

const TONEMAP_WGSL: &str = include_str!("tonemap.wgsl");

struct TonemapParams {
    buffer: wgpu::Buffer,
}

impl TonemapParams {
    fn new(device: &Device, srgb_target: bool) -> Self {
        // An sRGB render target applies the transfer function in hardware.
        let gamma: f32 = if srgb_target { 1.0 } else { 1.0 / 2.2 };
        let data = [
            1.0f32.to_le_bytes(),
            gamma.to_le_bytes(),
            [0u8; 4],
            [0u8; 4],
        ]
        .concat();
        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Orbital::TonemapParams"),
            contents: &data,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        Self { buffer }
    }
}

/// Owns the fullscreen tonemap pipeline and its static resources.
pub struct Tonemap {
    pipeline: RenderPipeline,
    bind_group_layout: BindGroupLayout,
    sampler: wgpu::Sampler,
    params: TonemapParams,
}

impl Tonemap {
    pub fn new(device: &Device, target_format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Orbital::TonemapShader"),
            source: ShaderSource::Wgsl(TONEMAP_WGSL.into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Orbital::TonemapBindGroupLayout"),
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
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Orbital::TonemapPipelineLayout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let targets = [Some(ColorTargetState {
            format: target_format,
            blend: Some(BlendState::REPLACE),
            write_mask: ColorWrites::ALL,
        })];

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Orbital::TonemapPipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &targets,
                compilation_options: Default::default(),
            }),
            depth_stencil: None,
            primitive: PrimitiveState::default(),
            cache: None,
            multisample: Default::default(),
            multiview_mask: None,
        });

        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("Orbital::TonemapSampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let params = TonemapParams::new(device, target_format.is_srgb());

        Self {
            pipeline,
            bind_group_layout,
            sampler,
            params,
        }
    }

    pub fn record(
        &self,
        device: &Device,
        encoder: &mut wgpu::CommandEncoder,
        hdr_view: &TextureView,
        target_view: &TextureView,
    ) {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Orbital::TonemapBindGroup"),
            layout: &self.bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(hdr_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(
                        self.params.buffer.as_entire_buffer_binding(),
                    ),
                },
            ],
        });

        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("RenderPass::Tonemap"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: target_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}

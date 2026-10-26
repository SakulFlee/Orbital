//! FXAA pass over the tonemapped LDR frame.

use wgpu::{
    BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
    BlendState, BufferBindingType, ColorTargetState, ColorWrites, Device, FragmentState,
    PipelineLayoutDescriptor, PrimitiveState, Queue, RenderPassColorAttachment,
    RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, SamplerBindingType,
    SamplerDescriptor, ShaderModuleDescriptor, ShaderSource, ShaderStages, StoreOp,
    TextureSampleType, TextureView, TextureViewDimension, VertexState,
};

const FXAA_WGSL: &str = include_str!("fxaa.wgsl");

pub struct Fxaa {
    pipeline: RenderPipeline,
    bind_group_layout: BindGroupLayout,
    sampler: wgpu::Sampler,
    params_buffer: wgpu::Buffer,
    resolution: cgmath::Vector2<u32>,
}

impl Fxaa {
    pub fn new(
        device: &Device,
        queue: &Queue,
        target_format: wgpu::TextureFormat,
        resolution: cgmath::Vector2<u32>,
    ) -> Self {
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Orbital::FxaaShader"),
            source: ShaderSource::Wgsl(FXAA_WGSL.into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Orbital::FxaaBindGroupLayout"),
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
            label: Some("Orbital::FxaaPipelineLayout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let targets = [Some(ColorTargetState {
            format: target_format,
            blend: Some(BlendState::REPLACE),
            write_mask: ColorWrites::ALL,
        })];

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Orbital::FxaaPipeline"),
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
            label: Some("Orbital::FxaaSampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Orbital::FxaaParams"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let fxaa = Self {
            pipeline,
            bind_group_layout,
            sampler,
            params_buffer,
            resolution,
        };
        fxaa.write_params(queue);
        fxaa
    }

    pub fn set_resolution(&mut self, queue: &Queue, resolution: cgmath::Vector2<u32>) {
        if self.resolution == resolution {
            return;
        }
        self.resolution = resolution;
        self.write_params(queue);
    }

    fn write_params(&self, queue: &Queue) {
        let inv_resolution = cgmath::Vector2::new(
            1.0 / self.resolution.x.max(1) as f32,
            1.0 / self.resolution.y.max(1) as f32,
        );
        let data = [
            inv_resolution.x.to_le_bytes(),
            inv_resolution.y.to_le_bytes(),
            [0u8; 4],
            [0u8; 4],
        ]
        .concat();
        queue.write_buffer(&self.params_buffer, 0, &data);
    }

    pub fn record(
        &self,
        device: &Device,
        encoder: &mut wgpu::CommandEncoder,
        source_view: &TextureView,
        target_view: &TextureView,
    ) {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Orbital::FxaaBindGroup"),
            layout: &self.bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(source_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(
                        self.params_buffer.as_entire_buffer_binding(),
                    ),
                },
            ],
        });

        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("RenderPass::Fxaa"),
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

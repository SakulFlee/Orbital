use wgpu::{
    BindGroupLayout, CompareFunction, DepthStencilState, Device, MultisampleState,
    PipelineLayoutDescriptor, PolygonMode, PrimitiveState, Queue, RenderPipeline,
    RenderPipelineDescriptor, SamplerDescriptor, ShaderStages, TextureFormat, TextureViewDimension,
    VertexState,
};

use crate::{Texture, Vertex};

use super::ShadowGpuData;

const SHADOW_DEPTH_SHADER: &str = include_str!("../../../../Assets/Shaders/shadow_depth.wgsl");

/// Manages shadow map rendering: depth texture array, uniform buffer, sampler, and depth pipeline.
pub struct ShadowRenderer {
    depth_texture: Texture,
    layer_count: u32,
    max_slots: u32,
    slot_data_buffer: wgpu::Buffer,
    sampler: wgpu::Sampler,
    depth_pipeline: RenderPipeline,
    depth_bind_group_layout: BindGroupLayout,
    gpu_data: ShadowGpuData,
    dirty: bool,
}

impl ShadowRenderer {
    pub fn new(device: &Device, _queue: &Queue, max_slots: u32, resolution: u32) -> Self {
        let initial_layers = 1u32.max(max_slots);

        let depth_texture = Self::create_depth_array_texture(device, resolution, initial_layers);

        let slot_data_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Shadow Slot Data Buffer"),
            size: std::mem::size_of::<ShadowGpuData>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("Shadow Map Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
            compare: Some(CompareFunction::LessEqual),
            anisotropy_clamp: 1,
            border_color: None,
        });

        let (depth_pipeline, depth_bind_group_layout) =
            Self::create_depth_pipeline(device, resolution);

        Self {
            depth_texture,
            layer_count: initial_layers,
            max_slots,
            slot_data_buffer,
            sampler,
            depth_pipeline,
            depth_bind_group_layout,
            gpu_data: ShadowGpuData::new(),
            dirty: false,
        }
    }

    fn create_depth_array_texture(device: &Device, resolution: u32, layers: u32) -> Texture {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Shadow Depth Array"),
            size: wgpu::Extent3d {
                width: resolution,
                height: resolution,
                depth_or_array_layers: layers,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Shadow Depth Array View"),
            dimension: Some(TextureViewDimension::D2Array),
            ..Default::default()
        });

        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("Shadow Depth Array Dummy Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });

        Texture::from_existing(texture, view, sampler, TextureViewDimension::D2Array)
    }

    fn create_depth_pipeline(
        device: &Device,
        resolution: u32,
    ) -> (RenderPipeline, BindGroupLayout) {
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shadow Depth Shader"),
            source: wgpu::ShaderSource::Wgsl(SHADOW_DEPTH_SHADER.into()),
        });

        // Bind group layout: just the light VP uniform
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Shadow Depth Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Shadow Depth Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Shadow Depth Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader_module,
                entry_point: Some("entrypoint_vertex"),
                buffers: &[
                    Some(Vertex::complex_vertex_buffer_layout_descriptor()),
                    Some(crate::Instance::vertex_buffer_layout_descriptor()),
                ],
                compilation_options: Default::default(),
            },
            fragment: None,
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(CompareFunction::LessEqual),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            primitive: PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            multisample: MultisampleState::default(),
            cache: None,
            multiview_mask: None,
        });

        (pipeline, bind_group_layout)
    }

    pub fn depth_texture(&self) -> &Texture {
        &self.depth_texture
    }

    pub fn slot_data_buffer(&self) -> &wgpu::Buffer {
        &self.slot_data_buffer
    }

    pub fn sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }

    pub fn gpu_data(&self) -> &ShadowGpuData {
        &self.gpu_data
    }

    pub fn gpu_data_mut(&mut self) -> &mut ShadowGpuData {
        &mut self.gpu_data
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn depth_pipeline(&self) -> &RenderPipeline {
        &self.depth_pipeline
    }

    pub fn depth_bind_group_layout(&self) -> &BindGroupLayout {
        &self.depth_bind_group_layout
    }

    /// Ensure the depth array has at least `needed` layers.
    /// Recreates the texture if needed. No shader recompilation required.
    pub fn ensure_layers(&mut self, device: &Device, resolution: u32, needed: u32) {
        if needed <= self.layer_count {
            return;
        }
        let new_layers = needed.max(self.layer_count * 2).min(self.max_slots);
        self.depth_texture = Self::create_depth_array_texture(device, resolution, new_layers);
        self.layer_count = new_layers;
        self.dirty = true;
    }

    /// Upload current GPU data to the uniform buffer.
    pub fn upload(&self, queue: &Queue) {
        queue.write_buffer(&self.slot_data_buffer, 0, self.gpu_data.as_bytes());
    }

    /// Mark data as needing upload.
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }
}

use wgpu::{
    CompareFunction, Device, Queue, RenderPipeline, SamplerDescriptor, TextureFormat,
    TextureViewDimension,
};

use crate::Texture;

use super::ShadowGpuData;

/// Manages shadow map rendering: depth texture array, uniform buffer, sampler, and depth pipeline.
pub struct ShadowRenderer {
    /// Depth texture array: 2D array of depth textures (one layer per shadow slot).
    depth_texture: Texture,
    /// Number of layers currently allocated in depth_texture.
    layer_count: u32,
    /// Maximum number of shadow slots (constant, e.g. 16).
    max_slots: u32,
    /// Uniform buffer containing all ShadowSlotData entries.
    slot_data_buffer: wgpu::Buffer,
    /// Comparison sampler for PCF shadow filtering.
    sampler: wgpu::Sampler,
    /// Depth-only render pipeline for rendering models into shadow maps.
    depth_pipeline: Option<RenderPipeline>,
    /// Current GPU data (CPU-side copy, uploaded each frame when dirty).
    gpu_data: ShadowGpuData,
    /// Dirty flag: set when slot data changes, triggers buffer upload.
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

        Self {
            depth_texture,
            layer_count: initial_layers,
            max_slots,
            slot_data_buffer,
            sampler,
            depth_pipeline: None, // Created in Phase 2
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

    pub fn depth_pipeline(&self) -> Option<&RenderPipeline> {
        self.depth_pipeline.as_ref()
    }

    pub fn set_depth_pipeline(&mut self, pipeline: RenderPipeline) {
        self.depth_pipeline = Some(pipeline);
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

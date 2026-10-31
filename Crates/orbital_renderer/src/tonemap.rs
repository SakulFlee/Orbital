//! Fullscreen HDR → LDR tonemap pass.

use cgmath::{Matrix4, Point3, Vector2};
use wgpu::{
    BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
    BlendState, BufferBindingType, ColorTargetState, ColorWrites, Device, FragmentState,
    PipelineLayoutDescriptor, PrimitiveState, RenderPassColorAttachment, RenderPassDescriptor,
    RenderPipeline, RenderPipelineDescriptor, SamplerBindingType, SamplerDescriptor,
    ShaderModuleDescriptor, ShaderSource, ShaderStages, StoreOp, TextureSampleType, TextureView,
    TextureViewDimension, VertexState,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::scene::{GlareLight, MAX_GLARE_LIGHTS};

const TONEMAP_WGSL: &str = include_str!("tonemap.wgsl");

/// Matrix (16) + camera position (4) + viewport (4) + lights (8 each).
const GLARE_PARAMS_FLOATS: usize = 16 + 4 + 4 + MAX_GLARE_LIGHTS * 8;
const GLARE_PARAMS_BYTES: usize = GLARE_PARAMS_FLOATS * 4;
const GLARE_RADIUS: f32 = 0.035;

struct TonemapParams {
    buffer: wgpu::Buffer,
    glare_buffer: wgpu::Buffer,
}

impl TonemapParams {
    fn new(device: &Device, srgb_target: bool) -> Self {
        // An sRGB render target applies the transfer function in hardware.
        let gamma: f32 = if srgb_target { 1.0 } else { 1.0 / 2.2 };
        let data = tonemap_params_data(1.0, gamma, 4.0, 0.0);
        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Orbital::TonemapParams"),
            contents: &data,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let glare_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Orbital::GlareParams"),
            contents: &[0u8; GLARE_PARAMS_BYTES],
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        Self {
            buffer,
            glare_buffer,
        }
    }
}

fn tonemap_params_data(
    exposure: f32,
    gamma: f32,
    bloom_threshold: f32,
    bloom_intensity: f32,
) -> [u8; 32] {
    let mut data = [0u8; 32];
    data[0..4].copy_from_slice(&exposure.to_le_bytes());
    data[4..8].copy_from_slice(&gamma.to_le_bytes());
    data[8..12].copy_from_slice(&bloom_threshold.to_le_bytes());
    data[12..16].copy_from_slice(&bloom_intensity.to_le_bytes());
    data
}

fn write_f32(data: &mut [u8], offset: usize, value: f32) {
    data[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

/// Owns the fullscreen tonemap pipeline and its static resources.
pub struct Tonemap {
    pipeline: RenderPipeline,
    bind_group_layout: BindGroupLayout,
    sampler: wgpu::Sampler,
    params: TonemapParams,
    gamma: f32,
    bind_group_cache: Option<(TextureView, TextureView, wgpu::BindGroup)>,
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
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Depth,
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 4,
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

        let gamma: f32 = if target_format.is_srgb() {
            1.0
        } else {
            1.0 / 2.2
        };
        let params = TonemapParams::new(device, target_format.is_srgb());

        Self {
            pipeline,
            bind_group_layout,
            sampler,
            params,
            gamma,
            bind_group_cache: None,
        }
    }

    /// Update exposure and bloom parameters for the next tonemap.
    pub fn set_params(
        &self,
        queue: &wgpu::Queue,
        exposure: f32,
        bloom_threshold: f32,
        bloom_intensity: f32,
    ) {
        let data = tonemap_params_data(exposure, self.gamma, bloom_threshold, bloom_intensity);
        queue.write_buffer(&self.params.buffer, 0, &data);
    }

    /// Update the camera and positional-light data used for screen-space glare.
    pub fn set_glare(
        &self,
        queue: &wgpu::Queue,
        view_projection: Matrix4<f32>,
        camera_position: Point3<f32>,
        resolution: Vector2<u32>,
        lights: &[GlareLight],
    ) {
        let mut data = [0u8; GLARE_PARAMS_BYTES];

        for (column, vector) in [
            view_projection.x,
            view_projection.y,
            view_projection.z,
            view_projection.w,
        ]
        .iter()
        .enumerate()
        {
            for (row, value) in [vector.x, vector.y, vector.z, vector.w].iter().enumerate() {
                write_f32(&mut data, (column * 4 + row) * 4, *value);
            }
        }

        write_f32(&mut data, 64, camera_position.x);
        write_f32(&mut data, 68, camera_position.y);
        write_f32(&mut data, 72, camera_position.z);

        write_f32(&mut data, 80, resolution.x as f32);
        write_f32(&mut data, 84, resolution.y as f32);
        write_f32(&mut data, 88, lights.len().min(MAX_GLARE_LIGHTS) as f32);
        write_f32(&mut data, 92, GLARE_RADIUS);

        for (index, light) in lights.iter().take(MAX_GLARE_LIGHTS).enumerate() {
            let offset = 96 + index * 32;
            write_f32(&mut data, offset, light.position.x);
            write_f32(&mut data, offset + 4, light.position.y);
            write_f32(&mut data, offset + 8, light.position.z);
            write_f32(&mut data, offset + 16, light.color.x);
            write_f32(&mut data, offset + 20, light.color.y);
            write_f32(&mut data, offset + 24, light.color.z);
            write_f32(&mut data, offset + 28, light.intensity);
        }

        queue.write_buffer(&self.params.glare_buffer, 0, &data);
    }

    pub fn record(
        &mut self,
        device: &Device,
        encoder: &mut wgpu::CommandEncoder,
        hdr_view: &TextureView,
        depth_view: &TextureView,
        target_view: &TextureView,
    ) {
        let cache_hit = self
            .bind_group_cache
            .as_ref()
            .is_some_and(|(view, depth, _)| view == hdr_view && depth == depth_view);
        if !cache_hit {
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
                    BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(depth_view),
                    },
                    BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::Buffer(
                            self.params.glare_buffer.as_entire_buffer_binding(),
                        ),
                    },
                ],
            });
            self.bind_group_cache = Some((hdr_view.clone(), depth_view.clone(), bind_group));
        }
        let bind_group = &self
            .bind_group_cache
            .as_ref()
            .expect("tonemap bind group cache")
            .2;

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
        pass.set_bind_group(0, bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tonemap_params_pack_exposure_and_bloom_in_uniform_layout() {
        let data = tonemap_params_data(2.0, 0.5, 0.75, 1.5);

        assert_eq!(data.len(), 32);
        assert_eq!(f32::from_le_bytes(data[0..4].try_into().unwrap()), 2.0);
        assert_eq!(f32::from_le_bytes(data[4..8].try_into().unwrap()), 0.5);
        assert_eq!(f32::from_le_bytes(data[8..12].try_into().unwrap()), 0.75);
        assert_eq!(f32::from_le_bytes(data[12..16].try_into().unwrap()), 1.5);
        assert!(data[16..].iter().all(|byte| *byte == 0));
    }
}

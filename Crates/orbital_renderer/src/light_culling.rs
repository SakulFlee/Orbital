//! Tiled light culling for the deferred lighting pass.
//!
//! A compute pass builds a per-tile list of light indices. The deferred
//! lighting shader then evaluates only the lights that can affect each tile.

use std::sync::Arc;

use cgmath::Vector2;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferBindingType,
    BufferDescriptor, BufferUsages, ComputePassDescriptor, ComputePipeline,
    ComputePipelineDescriptor, Device, ShaderModuleDescriptor, ShaderSource, ShaderStages,
    TextureSampleType, TextureView, TextureViewDimension,
};

use orbital_shader_preprocessor::{NodeRegistry, ShaderBuilder};

/// Screen tile edge length in pixels.
pub const TILE_SIZE: u32 = 16;
/// Upper bound of light indices stored per tile.
///
/// Tiles with more overlapping lights keep the first 128; the cost of a larger
/// fixed list grows linearly with the tile count.
pub const MAX_LIGHTS_PER_TILE: u32 = 128;

const WORKGROUP_SIZE: u32 = 8;
const PARAMS_BYTES: u64 = 16;

const LIGHT_CULLING_NODES: &[&str] = &["camera_uniform", "light_struct", "light_types"];

pub struct LightCulling {
    pipeline: ComputePipeline,
    tile_layout: BindGroupLayout,
    counts: Buffer,
    indices: Buffer,
    params: Buffer,
    tiles_x: u32,
    tiles_y: u32,
    bind_group_cache: Option<TileBindGroupCache>,
}

struct TileBindGroupCache {
    depth: TextureView,
    counts: Buffer,
    indices: Buffer,
    params: Buffer,
    bind_group: BindGroup,
}

impl LightCulling {
    pub fn new(
        device: &Device,
        queue: &wgpu::Queue,
        resolution: Vector2<u32>,
        world_layout: &BindGroupLayout,
    ) -> Self {
        let mut registry = NodeRegistry::new();
        registry
            .register_library(&orbital_shader_engine::engine_library())
            .expect("engine node library");

        let mut builder = ShaderBuilder::new(Arc::new(registry));
        builder
            .add_nodes(LIGHT_CULLING_NODES)
            .expect("light culling nodes");
        builder.add_source(include_str!("light_culling.wgsl"));
        let source = builder.build();

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Orbital::LightCullingShader"),
            source: ShaderSource::Wgsl(source.into()),
        });

        let tile_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Orbital::LightCullingTileLayout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Depth,
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Orbital::LightCullingPipelineLayout"),
            bind_group_layouts: &[Some(world_layout), Some(&tile_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Orbital::LightCullingPipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        let tiles_x = tile_count(resolution.x);
        let tiles_y = tile_count(resolution.y);
        let (counts, indices, params) = create_buffers(device, queue, tiles_x, tiles_y);

        Self {
            pipeline,
            tile_layout,
            counts,
            indices,
            params,
            tiles_x,
            tiles_y,
            bind_group_cache: None,
        }
    }

    /// Reallocate tile buffers when the render resolution changes.
    pub fn ensure(&mut self, device: &Device, queue: &wgpu::Queue, resolution: Vector2<u32>) {
        let tiles_x = tile_count(resolution.x);
        let tiles_y = tile_count(resolution.y);
        if tiles_x == self.tiles_x && tiles_y == self.tiles_y {
            return;
        }

        let (counts, indices, params) = create_buffers(device, queue, tiles_x, tiles_y);
        self.counts = counts;
        self.indices = indices;
        self.params = params;
        self.tiles_x = tiles_x;
        self.tiles_y = tiles_y;
        self.bind_group_cache = None;
    }

    pub fn record(
        &mut self,
        device: &Device,
        encoder: &mut wgpu::CommandEncoder,
        world_bind_group: &BindGroup,
        depth_view: &TextureView,
    ) {
        let cache_hit = self.bind_group_cache.as_ref().is_some_and(|cache| {
            cache.depth == *depth_view
                && cache.counts == self.counts
                && cache.indices == self.indices
                && cache.params == self.params
        });

        if !cache_hit {
            let bind_group = device.create_bind_group(&BindGroupDescriptor {
                label: Some("Orbital::LightCullingTileBindGroup"),
                layout: &self.tile_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(depth_view),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: self.counts.as_entire_binding(),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: self.indices.as_entire_binding(),
                    },
                    BindGroupEntry {
                        binding: 3,
                        resource: self.params.as_entire_binding(),
                    },
                ],
            });
            self.bind_group_cache = Some(TileBindGroupCache {
                depth: depth_view.clone(),
                counts: self.counts.clone(),
                indices: self.indices.clone(),
                params: self.params.clone(),
                bind_group,
            });
        }

        let bind_group = &self
            .bind_group_cache
            .as_ref()
            .expect("light culling bind group cache was filled")
            .bind_group;

        let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("Light Culling"),
            ..Default::default()
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, world_bind_group, &[]);
        pass.set_bind_group(1, bind_group, &[]);
        pass.dispatch_workgroups(
            self.tiles_x.div_ceil(WORKGROUP_SIZE),
            self.tiles_y.div_ceil(WORKGROUP_SIZE),
            1,
        );
    }

    pub fn counts(&self) -> &Buffer {
        &self.counts
    }

    pub fn indices(&self) -> &Buffer {
        &self.indices
    }

    pub fn params(&self) -> &Buffer {
        &self.params
    }
}

fn tile_count(size: u32) -> u32 {
    size.max(1).div_ceil(TILE_SIZE)
}

fn create_buffers(
    device: &Device,
    queue: &wgpu::Queue,
    tiles_x: u32,
    tiles_y: u32,
) -> (Buffer, Buffer, Buffer) {
    let tile_count = u64::from(tiles_x) * u64::from(tiles_y);
    let counts = device.create_buffer(&BufferDescriptor {
        label: Some("Orbital::TileLightCounts"),
        size: tile_count * 4,
        usage: BufferUsages::STORAGE,
        mapped_at_creation: false,
    });
    let indices = device.create_buffer(&BufferDescriptor {
        label: Some("Orbital::TileLightIndices"),
        size: tile_count * u64::from(MAX_LIGHTS_PER_TILE) * 4,
        usage: BufferUsages::STORAGE,
        mapped_at_creation: false,
    });
    let params = device.create_buffer(&BufferDescriptor {
        label: Some("Orbital::TileLightParams"),
        size: PARAMS_BYTES,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let values = [tiles_x, tiles_y, MAX_LIGHTS_PER_TILE, TILE_SIZE];
    let mut bytes = [0u8; PARAMS_BYTES as usize];
    for (index, value) in values.iter().enumerate() {
        bytes[index * 4..index * 4 + 4].copy_from_slice(&value.to_le_bytes());
    }
    queue.write_buffer(&params, 0, &bytes);

    (counts, indices, params)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tile_count_rounds_up_and_handles_empty_sizes() {
        assert_eq!(tile_count(0), 1);
        assert_eq!(tile_count(1), 1);
        assert_eq!(tile_count(16), 1);
        assert_eq!(tile_count(17), 2);
        assert_eq!(tile_count(1920), 120);
        assert_eq!(tile_count(1080), 68);
    }
}

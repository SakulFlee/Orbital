use std::num::NonZero;

use cgmath::{Deg, InnerSpace, Matrix4, Point3, Rad, SquareMatrix, Vector3, Vector4};
use wgpu::{
    BindGroup, BindGroupLayout, CompareFunction, DepthStencilState, Device,
    MultisampleState, PipelineLayoutDescriptor, PolygonMode, PrimitiveState, Queue,
    RenderPipeline, RenderPipelineDescriptor, SamplerDescriptor, ShaderStages, TextureFormat,
    TextureViewDimension, VertexState,
};

use crate::projection::{ortho_wgpu, perspective_wgpu};
use crate::{Texture, Vertex};

use super::{
    ShadowGpuData, ShadowLightInfo, ShadowSlotData,
    SHADOW_TYPE_DIRECTIONAL_CASCADE, SHADOW_TYPE_POINT, SHADOW_TYPE_SPOT,
};

const SHADOW_DEPTH_SHADER: &str = include_str!("../../../../Assets/Shaders/shadow_depth.wgsl");

/// Result of a single cascade computation.
struct CascadeInfo {
    vp: Matrix4<f32>,
    split_depth: f32,
}

const POINT_LIGHT_FAR: f32 = 20.0;
const POINT_LIGHT_NEAR: f32 = 0.1;
/// Spot shadow map clip planes. The far plane is intentionally tight:
/// beyond ~50 units the spot attenuation is already negligible
/// (1/distance² at 50 units = 1/2500), so capturing geometry that far
/// eats depth precision for no visual gain.  A generous near plane
/// avoids the worst non-linearity right at the light source.
const SPOT_SHADOW_FAR: f32 = 50.0;
const SPOT_SHADOW_NEAR: f32 = 1.0;
/// Depth at which the spot shadow switches from the near map to the
/// far map. The near map captures the floor and close geometry; the
/// far map captures walls and distant geometry. This split prevents
/// the floor and wall from overlapping in shadow space.
const SPOT_SPLIT_DEPTH: f32 = 8.0;

const CUBE_FACE_DIRECTIONS: [(Vector3<f32>, Vector3<f32>); 6] = [
    (Vector3::new(1.0, 0.0, 0.0), Vector3::new(0.0, -1.0, 0.0)),  // +X
    (Vector3::new(-1.0, 0.0, 0.0), Vector3::new(0.0, -1.0, 0.0)), // -X
    (Vector3::new(0.0, 1.0, 0.0), Vector3::new(0.0, 0.0, 1.0)),   // +Y
    (Vector3::new(0.0, -1.0, 0.0), Vector3::new(0.0, 0.0, -1.0)), // -Y
    (Vector3::new(0.0, 0.0, 1.0), Vector3::new(0.0, -1.0, 0.0)),  // +Z
    (Vector3::new(0.0, 0.0, -1.0), Vector3::new(0.0, -1.0, 0.0)), // -Z
];

/// Manages shadow map rendering: depth texture array, uniform buffer, sampler, and depth pipeline.
pub struct ShadowRenderer {
    depth_texture: Texture,
    layer_count: u32,
    max_slots: u32,
    resolution: u32,
    /// Cube depth texture array for point light shadows.
    cube_depth_texture: Texture,
    cube_count: u32,
    slot_data_buffer: wgpu::Buffer,
    sampler: wgpu::Sampler,
    /// Comparison sampler for point light cube shadows.
    cube_sampler: wgpu::Sampler,
    depth_pipeline: RenderPipeline,
    depth_bind_group_layout: BindGroupLayout,
    /// Per-slot matrix buffer (dynamic offset), reused every frame.
    matrix_buffer: wgpu::Buffer,
    /// Bind group for per-slot matrix (dynamic offset).
    matrix_bind_group: BindGroup,
    /// Stride per slot in the matrix buffer (aligned to min_uniform_buffer_offset_alignment).
    slot_stride: u64,
    gpu_data: ShadowGpuData,
}

impl ShadowRenderer {
    pub fn new(device: &Device, queue: &Queue, max_slots: u32, resolution: u32) -> Self {
        let initial_layers = 1u32.max(max_slots);

        let depth_texture = Self::create_depth_array_texture(device, resolution, initial_layers);

        let initial_gpu = ShadowGpuData::new();
        let slot_data_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Shadow Slot Data Buffer"),
            size: std::mem::size_of::<ShadowGpuData>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        // Initialize with cascade_count = 0 so first frame doesn't read garbage
        queue.write_buffer(&slot_data_buffer, 0, initial_gpu.as_bytes());

        // Linear filtering on a comparison sampler gives bilinear PCF per tap.
        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("Shadow Map Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
            compare: Some(CompareFunction::LessEqual),
            anisotropy_clamp: 1,
            border_color: None,
        });

        // Point light cube shadow array (initially 1 cube = 6 layers)
        let cube_depth_texture = Self::create_cube_depth_texture(device, resolution, 1);
        let cube_sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("Shadow Cube Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
            compare: Some(CompareFunction::LessEqual),
            anisotropy_clamp: 1,
            border_color: None,
        });

        let (depth_pipeline, depth_bind_group_layout) =
            Self::create_depth_pipeline(device);

        // Per-slot matrix buffer (dynamic offset)
        let alignment = device.limits().min_uniform_buffer_offset_alignment;
        let slot_stride = alignment.max(64) as u64;
        let matrix_buffer_size = slot_stride * max_slots as u64 * 6;
        let matrix_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Shadow Per-Slot Matrix Buffer"),
            size: matrix_buffer_size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let matrix_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Shadow Depth Matrix Bind Group"),
            layout: &depth_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &matrix_buffer,
                    offset: 0,
                    size: NonZero::new(64),
                }),
            }],
        });

        Self {
            depth_texture,
            layer_count: initial_layers,
            max_slots,
            resolution,
            cube_depth_texture,
            cube_count: 1,
            slot_data_buffer,
            sampler,
            cube_sampler,
            depth_pipeline,
            depth_bind_group_layout,
            matrix_buffer,
            matrix_bind_group,
            slot_stride,
            gpu_data: ShadowGpuData::new(),
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
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Shadow Depth Array View"),
            dimension: Some(TextureViewDimension::D2Array),
            ..Default::default()
        });

        let dummy_sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("Shadow Depth Array Dummy Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });

        Texture::from_existing(texture, view, dummy_sampler, TextureViewDimension::D2Array)
    }

    fn create_cube_depth_texture(device: &Device, resolution: u32, cubes: u32) -> Texture {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Shadow Cube Depth Array"),
            size: wgpu::Extent3d {
                width: resolution,
                height: resolution,
                depth_or_array_layers: cubes * 6,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Shadow Cube Depth Array View"),
            dimension: Some(TextureViewDimension::CubeArray),
            ..Default::default()
        });

        let dummy_sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("Shadow Cube Depth Array Dummy Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });

        Texture::from_existing(texture, view, dummy_sampler, TextureViewDimension::CubeArray)
    }

    fn create_depth_pipeline(device: &Device) -> (RenderPipeline, BindGroupLayout) {
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shadow Depth Shader"),
            source: wgpu::ShaderSource::Wgsl(SHADOW_DEPTH_SHADER.into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Shadow Depth Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
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
                bias: wgpu::DepthBiasState {
                    constant: 0,
                    slope_scale: 0.0,
                    clamp: 0.0,
                },
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

    /// Render all shadow maps for the current frame.
    ///
    /// `camera_perspective_view_proj` is the camera's combined perspective × view matrix.
    /// `camera_near` / `camera_far` are the camera's clip planes (for cascade splits).
    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        models: &[&crate::Model],
        shadow_lights: &[ShadowLightInfo],
        camera_perspective_view_proj: &Matrix4<f32>,
        camera_near: f32,
        camera_far: f32,
        device: &Device,
        queue: &Queue,
    ) {
        self.gpu_data = ShadowGpuData::new();

        let mut slot_index = 0u32;
        let mut layer_index = 0u32;
        let mut cube_index = 0u32;
        let mut matrix_index = 0u32;
        let matrix_buf_size = self.slot_stride * self.max_slots as u64 * 6;
        let mut matrix_bytes = vec![0u8; matrix_buf_size as usize];
        // Maps shadow slot -> index into the matrix buffer (point lights
        // consume 6 matrix entries, everything else 1).
        let mut slot_matrix_offsets: Vec<u32> = Vec::with_capacity(self.max_slots as usize);

        for light in shadow_lights {
            if !light.caster.enabled {
                continue;
            }
            if slot_index >= self.max_slots {
                log::warn!("Shadow slot limit ({}) reached, dropping remaining lights", self.max_slots);
                break;
            }

            let _resolution = light.caster.resolution.max(1);

            match light.light_type {
                0 => {
                    // Point light — cube shadow map (6 faces)
                    if slot_index >= self.max_slots {
                        break;
                    }
                    let my_cube = cube_index;
                    self.ensure_cubes(device, my_cube + 1);

                    let pos = Point3::new(
                        light.position.x,
                        light.position.y,
                        light.position.z,
                    );
                    let far_plane = POINT_LIGHT_FAR;
                    let near_plane = POINT_LIGHT_NEAR;
                    let shadow_type = SHADOW_TYPE_POINT;
                    let face_mats = Self::point_light_face_matrices(pos, near_plane, far_plane);

                    slot_matrix_offsets.push(matrix_index);

                    for (face, mat) in face_mats.iter().enumerate() {
                        let off = (matrix_index as usize + face as usize) * self.slot_stride as usize;
                        let vp_bytes = matrix_to_bytes(mat);
                        let end = (off + 64).min(matrix_bytes.len());
                        matrix_bytes[off..end].copy_from_slice(&vp_bytes[..end - off]);
                    }

                    let mut slot_data = ShadowSlotData {
                        light_view_proj: [[0.0; 4]; 4],
                        shadow_type,
                        layer_index: my_cube,
                        cascade_split_depth: far_plane,
                        bias: light.caster.bias,
                        light_index: light.light_store_index,
                        near_plane,
                        _padding: [0; 2],
                    };
                    slot_data.light_view_proj[3] = [
                        light.position.x,
                        light.position.y,
                        light.position.z,
                        1.0,
                    ];
                    self.gpu_data.slots[slot_index as usize] = slot_data;

                    for face in 0..6 {
                        let face_layer = my_cube * 6 + face;
                        let face_view = self.cube_depth_texture.texture().create_view(
                            &wgpu::TextureViewDescriptor {
                                label: Some("Shadow Cube Face View"),
                                dimension: Some(TextureViewDimension::D2),
                                base_array_layer: face_layer,
                                array_layer_count: Some(1),
                                ..Default::default()
                            },
                        );

                        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("Shadow Cube Face Render Pass"),
                            color_attachments: &[],
                            depth_stencil_attachment: Some(
                                wgpu::RenderPassDepthStencilAttachment {
                                    view: &face_view,
                                    depth_ops: Some(wgpu::Operations {
                                        load: wgpu::LoadOp::Clear(1.0),
                                        store: wgpu::StoreOp::Store,
                                    }),
                                    stencil_ops: None,
                                },
                            ),
                            timestamp_writes: None,
                            occlusion_query_set: None,
                            multiview_mask: None,
                        });

                        pass.set_pipeline(&self.depth_pipeline);
                        pass.set_bind_group(
                            0,
                            &self.matrix_bind_group,
                            &[((matrix_index as usize + face as usize) as u64 * self.slot_stride) as u32],
                        );

                        for model in models {
                            pass.set_vertex_buffer(0, model.mesh().vertex_buffer().slice(..));
                            pass.set_vertex_buffer(1, model.instance_buffer().slice(..));
                            pass.set_index_buffer(
                                model.mesh().index_buffer().slice(..),
                                wgpu::IndexFormat::Uint32,
                            );
                            pass.draw_indexed(
                                0..model.mesh().index_count(),
                                0,
                                0..model.instance_count(),
                            );
                        }
                    }

                    slot_index += 1;
                    cube_index += 1;
                    matrix_index += 6;
                }
                1 => {
                    // Directional light → CSM
                    let cascades = compute_csm_cascades(
                        camera_perspective_view_proj,
                        light.direction,
                        camera_near,
                        camera_far,
                        light.caster.cascade_count.max(1),
                        light.caster.cascade_split_lambda,
                    );

                    for cascade in &cascades {
                        if slot_index >= self.max_slots {
                            break;
                        }
                        self.gpu_data.slots[slot_index as usize] = ShadowSlotData {
                            light_view_proj: cascade.vp.into(),
                            shadow_type: SHADOW_TYPE_DIRECTIONAL_CASCADE,
                            layer_index,
                            cascade_split_depth: cascade.split_depth,
                            bias: light.caster.bias,
                            light_index: light.light_store_index,
                            near_plane: 0.1,
                            _padding: [0; 2],
                        };

                        slot_matrix_offsets.push(matrix_index);
                        let offset = matrix_index as u64 * self.slot_stride;
                        let vp_bytes = matrix_to_bytes(&cascade.vp);
                        matrix_bytes[offset as usize..offset as usize + 64]
                            .copy_from_slice(&vp_bytes);

                        slot_index += 1;
                        layer_index += 1;
                        matrix_index += 1;
                    }
                }
                2 => {
                    // Spot light — dual-range perspective maps.
                    // Near map (1.0–SPLIT): captures floor and close geometry.
                    // Far  map (SPLIT–FAR): captures walls and distant geometry.
                    // The split prevents floor and wall from overlapping in
                    // shadow space, which would cause false self-occlusion.
                    let vp_near = Self::spot_light_vp(
                        light.position, light.direction,
                        light.outer_cone_angle,
                        SPOT_SHADOW_NEAR, SPOT_SPLIT_DEPTH,
                    );
                    let vp_far = Self::spot_light_vp(
                        light.position, light.direction,
                        light.outer_cone_angle,
                        SPOT_SPLIT_DEPTH, SPOT_SHADOW_FAR,
                    );

                    // --- Near slot ---
                    self.gpu_data.slots[slot_index as usize] = ShadowSlotData {
                        light_view_proj: vp_near.into(),
                        shadow_type: SHADOW_TYPE_SPOT,
                        layer_index,
                        cascade_split_depth: SPOT_SPLIT_DEPTH, // gate
                        bias: light.caster.bias,
                        light_index: light.light_store_index,
                        near_plane: SPOT_SHADOW_NEAR,
                        _padding: [0; 2],
                    };
                    slot_matrix_offsets.push(matrix_index);
                    let off = matrix_index as usize * self.slot_stride as usize;
                    matrix_bytes[off..off + 64].copy_from_slice(&matrix_to_bytes(&vp_near));
                    slot_index += 1; layer_index += 1; matrix_index += 1;

                    // --- Far slot ---
                    self.gpu_data.slots[slot_index as usize] = ShadowSlotData {
                        light_view_proj: vp_far.into(),
                        shadow_type: SHADOW_TYPE_SPOT,
                        layer_index,
                        cascade_split_depth: SPOT_SHADOW_FAR, // gate
                        bias: light.caster.bias,
                        light_index: light.light_store_index,
                        near_plane: SPOT_SPLIT_DEPTH,
                        _padding: [0; 2],
                    };
                    slot_matrix_offsets.push(matrix_index);
                    let off = matrix_index as usize * self.slot_stride as usize;
                    matrix_bytes[off..off + 64].copy_from_slice(&matrix_to_bytes(&vp_far));
                    slot_index += 1; layer_index += 1; matrix_index += 1;
                }
                _ => {}
            }
        }

        self.gpu_data.cascade_count = slot_index;

        // Ensure enough layers in the depth texture
        if layer_index > 0 {
            self.ensure_layers(device, layer_index);
        }

        // Upload slot data
        queue.write_buffer(&self.slot_data_buffer, 0, self.gpu_data.as_bytes());

        // Upload per-slot matrices
        queue.write_buffer(&self.matrix_buffer, 0, &matrix_bytes);

        // Render each slot that targets the 2D depth array
        // (directional cascades + spot lights; point lights rendered inline above)
        for i in 0..slot_index {
            let slot = &self.gpu_data.slots[i as usize];
            if slot.shadow_type == SHADOW_TYPE_POINT {
                continue;
            }

            log::debug!(
                "Depth pass slot {}: type={} layer={} models={}",
                i,
                if slot.shadow_type == SHADOW_TYPE_DIRECTIONAL_CASCADE {
                    "CSM"
                } else {
                    "SPOT"
                },
                slot.layer_index,
                models.len(),
            );

            let layer_view = self.depth_texture.texture().create_view(
                &wgpu::TextureViewDescriptor {
                    label: Some("Shadow Layer View"),
                    dimension: Some(TextureViewDimension::D2Array),
                    base_array_layer: slot.layer_index,
                    array_layer_count: Some(1),
                    ..Default::default()
                },
            );

            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Shadow Map Render Pass"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &layer_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            pass.set_pipeline(&self.depth_pipeline);
            pass.set_bind_group(
                0,
                &self.matrix_bind_group,
                &[(slot_matrix_offsets[i as usize] as u64 * self.slot_stride) as u32],
            );

            for model in models {
                pass.set_vertex_buffer(0, model.mesh().vertex_buffer().slice(..));
                pass.set_vertex_buffer(1, model.instance_buffer().slice(..));
                pass.set_index_buffer(
                    model.mesh().index_buffer().slice(..),
                    wgpu::IndexFormat::Uint32,
                );
                pass.draw_indexed(
                    0..model.mesh().index_count(),
                    0,
                    0..model.instance_count(),
                );
            }
        }
    }

    // --- Accessors ---

    pub fn depth_texture(&self) -> &Texture {
        &self.depth_texture
    }

    pub fn slot_data_buffer(&self) -> &wgpu::Buffer {
        &self.slot_data_buffer
    }

    pub fn sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }

    pub fn cube_depth_texture(&self) -> &Texture {
        &self.cube_depth_texture
    }

    pub fn cube_sampler(&self) -> &wgpu::Sampler {
        &self.cube_sampler
    }

    pub fn gpu_data(&self) -> &ShadowGpuData {
        &self.gpu_data
    }

    pub fn depth_pipeline(&self) -> &RenderPipeline {
        &self.depth_pipeline
    }

    pub fn depth_bind_group_layout(&self) -> &BindGroupLayout {
        &self.depth_bind_group_layout
    }

    /// Ensure the depth array has at least `needed` layers.
    pub fn ensure_layers(&mut self, device: &Device, needed: u32) {
        if needed <= self.layer_count {
            return;
        }
        let new_layers = needed.max(self.layer_count * 2).min(self.max_slots);
        self.depth_texture = Self::create_depth_array_texture(device, self.resolution, new_layers);
        self.layer_count = new_layers;
    }

    /// Ensure the cube depth array has at least `needed` cubes.
    pub fn ensure_cubes(&mut self, device: &Device, needed: u32) {
        if needed <= self.cube_count {
            return;
        }
        let new_cubes = needed.max(self.cube_count * 2).min(self.max_slots);
        self.cube_depth_texture =
            Self::create_cube_depth_texture(device, self.resolution, new_cubes);
        self.cube_count = new_cubes;
    }

    /// Upload current GPU data to the uniform buffer.
    pub fn upload(&self, queue: &Queue) {
        queue.write_buffer(&self.slot_data_buffer, 0, self.gpu_data.as_bytes());
    }
}

// ---------------------------------------------------------------------------
// CSM Helpers
// ---------------------------------------------------------------------------

/// Compute the 8 corners of the view frustum in world space from the
/// inverse of the combined perspective × view matrix.
///
/// Uses the wgpu clip convention: the near plane is at z_ndc = 0.
fn compute_frustum_corners_world(inv_view_proj: &Matrix4<f32>) -> [Point3<f32>; 8] {
    let corners_ndc = [
        Vector4::new(-1.0, -1.0, 0.0, 1.0),
        Vector4::new(1.0, -1.0, 0.0, 1.0),
        Vector4::new(-1.0, 1.0, 0.0, 1.0),
        Vector4::new(1.0, 1.0, 0.0, 1.0),
        Vector4::new(-1.0, -1.0, 1.0, 1.0),
        Vector4::new(1.0, -1.0, 1.0, 1.0),
        Vector4::new(-1.0, 1.0, 1.0, 1.0),
        Vector4::new(1.0, 1.0, 1.0, 1.0),
    ];

    let mut world = [Point3::new(0.0, 0.0, 0.0); 8];
    for (i, ndc) in corners_ndc.iter().enumerate() {
        let clip = inv_view_proj * ndc;
        world[i] = Point3::new(clip.x / clip.w, clip.y / clip.w, clip.z / clip.w);
    }
    world
}

/// Compute CSM cascade splits using the practical split scheme.
/// Blends between uniform and logarithmic splits.
fn compute_csm_cascades(
    camera_view_proj: &Matrix4<f32>,
    light_direction: Vector3<f32>,
    near: f32,
    far: f32,
    cascade_count: u32,
    lambda: f32,
) -> Vec<CascadeInfo> {
    let inv = camera_view_proj.invert().unwrap_or(Matrix4::identity());
    let frustum_corners = compute_frustum_corners_world(&inv);

    // Compute split depths
    let mut split_depths = vec![near];
    for i in 1..cascade_count {
        let i_f = i as f32;
        let n = cascade_count as f32;

        // Logarithmic split
        let log_split = near * (far / near).powf(i_f / n);
        // Uniform split
        let uniform_split = near + (far - near) * (i_f / n);
        // Blend
        let d = lambda * log_split + (1.0 - lambda) * uniform_split;
        split_depths.push(d);
    }
    split_depths.push(far);

    let mut cascades = Vec::with_capacity(cascade_count as usize);

    for i in 0..cascade_count as usize {
        let d_min = split_depths[i];
        let d_max = split_depths[i + 1];

        // Interpolate frustum edges at the cascade boundaries
        let near_corners = interpolate_frustum_edges(&frustum_corners, d_min / far);
        let far_corners = interpolate_frustum_edges(&frustum_corners, d_max / far);

        // Build the bounding box in light space
        let light_dir = light_direction.normalize();
        let light_up = if light_dir.y.abs() < 0.9 {
            Vector3::unit_y()
        } else {
            Vector3::unit_z()
        };
        let light_right = light_dir.cross(light_up).normalize();
        let light_up_actual = light_right.cross(light_dir).normalize();

        let mut min_bb = Vector3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut max_bb = Vector3::new(f32::MIN, f32::MIN, f32::MIN);

        for corner in near_corners.iter().chain(far_corners.iter()) {
            let rel = Vector3::new(corner.x, corner.y, corner.z);
            let lx = light_right.dot(rel);
            let ly = light_up_actual.dot(rel);
            let lz = light_dir.dot(rel);

            min_bb.x = min_bb.x.min(lx);
            max_bb.x = max_bb.x.max(lx);
            min_bb.y = min_bb.y.min(ly);
            max_bb.y = max_bb.y.max(ly);
            min_bb.z = min_bb.z.min(lz);
            max_bb.z = max_bb.z.max(lz);
        }

        // Light view matrix: look from the center of the cascade, along light direction
        let cascade_center = Vector3::new(
            (max_bb.x + min_bb.x) * 0.5,
            (max_bb.y + min_bb.y) * 0.5,
            (max_bb.z + min_bb.z) * 0.5,
        );
        let light_pos = Point3::new(
            cascade_center.x + light_dir.x * max_bb.z * 2.0,
            cascade_center.y + light_dir.y * max_bb.z * 2.0,
            cascade_center.z + light_dir.z * max_bb.z * 2.0,
        );
        let cascade_center_pt = Point3::new(cascade_center.x, cascade_center.y, cascade_center.z);
        let view = Matrix4::look_at_rh(light_pos, cascade_center_pt, light_up_actual);

        // Orthographic projection from the bounding box (wgpu clip convention,
        // Y is flipped in the shader sampling, not the projection).
        let proj = ortho_wgpu(
            min_bb.x,
            max_bb.x,
            min_bb.y,
            max_bb.y,
            0.0,
            (max_bb.z * 3.0).max(1.0),
            false,
        );

        cascades.push(CascadeInfo {
            vp: proj * view,
            split_depth: d_max,
        });
    }

    cascades
}

/// Interpolate frustum edges from near (t=0) to far (t=1).
fn interpolate_frustum_edges(corners: &[Point3<f32>; 8], t: f32) -> [Point3<f32>; 4] {
    [
        Point3::new(
            corners[0].x + (corners[4].x - corners[0].x) * t,
            corners[0].y + (corners[4].y - corners[0].y) * t,
            corners[0].z + (corners[4].z - corners[0].z) * t,
        ),
        Point3::new(
            corners[1].x + (corners[5].x - corners[1].x) * t,
            corners[1].y + (corners[5].y - corners[1].y) * t,
            corners[1].z + (corners[5].z - corners[1].z) * t,
        ),
        Point3::new(
            corners[2].x + (corners[6].x - corners[2].x) * t,
            corners[2].y + (corners[6].y - corners[2].y) * t,
            corners[2].z + (corners[6].z - corners[2].z) * t,
        ),
        Point3::new(
            corners[3].x + (corners[7].x - corners[3].x) * t,
            corners[3].y + (corners[7].y - corners[3].y) * t,
            corners[3].z + (corners[7].z - corners[3].z) * t,
        ),
    ]
}

impl ShadowRenderer {
    /// Compute the view-projection matrix for a spot light shadow map.
    ///
    /// A single perspective projection covering exactly the outer cone of the
    /// light, rendered into one layer of the 2D shadow depth array. Uses the
    /// wgpu clip convention with a Y flip so the shader can sample the map
    /// with `ndc.xy * 0.5 + 0.5`.
    fn spot_light_vp(
        position: Vector3<f32>,
        direction: Vector3<f32>,
        outer_cone_angle: f32,
        near: f32,
        far: f32,
    ) -> Matrix4<f32> {
        let dir = if direction.magnitude2() > 1e-12 {
            direction.normalize()
        } else {
            Vector3::new(0.0, -1.0, 0.0)
        };
        let up = if dir.y.abs() < 0.99 {
            Vector3::unit_y()
        } else {
            Vector3::unit_x()
        };
        let pos = Point3::new(position.x, position.y, position.z);
        let view = Matrix4::look_at_rh(pos, pos + dir, up);
        let fov = Rad((outer_cone_angle * 2.0 * 1.02).clamp(0.05, 2.96));
        let proj = perspective_wgpu(fov, 1.0, near, far, false);
        proj * view
    }

    /// Compute the 6 face view-projection matrices for a cube shadow map.
    fn point_light_face_matrices(position: Point3<f32>, near_plane: f32, far_plane: f32) -> [Matrix4<f32>; 6] {
        // wgpu clip convention; Y flip matches the cube-face framebuffer layout.
        let proj = perspective_wgpu(Rad::from(Deg(90.0)), 1.0, near_plane, far_plane, false);
        let mut mats = [Matrix4::identity(); 6];
        for (i, &(dir, up)) in CUBE_FACE_DIRECTIONS.iter().enumerate() {
            let target = position + dir;
            let view = Matrix4::look_at_rh(position, target, up);
            mats[i] = proj * view;
        }
        mats
    }
}

/// Convert a Matrix4<f32> to 64 bytes for GPU upload.
fn matrix_to_bytes(matrix: &Matrix4<f32>) -> [u8; 64] {
    // cgmath Matrix4 is column-major: components accessed as .x, .y, .z, .w
    // where .x is the first column (Vector4 of rows), and x.x is row 0 col 0.
    // WGSL expects column-major, so we just flatten columns in order.
    let mut bytes = [0u8; 64];
    let cols = [matrix.x, matrix.y, matrix.z, matrix.w];
    for (col_idx, col) in cols.iter().enumerate() {
        let row_data = [col.x, col.y, col.z, col.w];
        for (row_idx, val) in row_data.iter().enumerate() {
            let offset = (col_idx * 4 + row_idx) * 4;
            bytes[offset..offset + 4].copy_from_slice(&val.to_le_bytes());
        }
    }
    bytes
}

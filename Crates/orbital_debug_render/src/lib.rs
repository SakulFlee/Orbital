use cgmath::{Matrix4, Point3, SquareMatrix, Vector4};
use orbital_app::{Module, RenderOverlay, RenderOverlayContext, RenderOverlayResource};
use orbital_ecs::{IntoSystem, Res, ResMut, System, World};
use orbital_ecs_bridge::{
    ActiveCamera, EcsCameraStore, InputSnapshot, ModelInstances, ModelRealization,
};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, BlendComponent, BlendFactor, BlendOperation, BlendState,
    Buffer, BufferBindingType, BufferDescriptor, BufferUsages, ColorTargetState, ColorWrites,
    CompareFunction, DepthStencilState, Device, FragmentState,
    MultisampleState, PipelineLayoutDescriptor, PrimitiveState, PrimitiveTopology,
    Queue, RenderPass, RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor,
    ShaderSource, ShaderStages, TextureFormat, VertexAttribute, VertexBufferLayout,
    VertexFormat, VertexState, VertexStepMode,
};
use winit::keyboard::{KeyCode, PhysicalKey};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const SPHERE_SEGMENTS: u32 = 32;
const SPHERE_RINGS: u32 = 3;
const VERTS_PER_SPHERE: u32 = SPHERE_RINGS * SPHERE_SEGMENTS * 2;
const MAX_SPHERE_INSTANCES: u32 = 512;
const FRUSTUM_VERTS: u32 = 24;
const MAX_VERTS: u32 = MAX_SPHERE_INSTANCES * VERTS_PER_SPHERE + FRUSTUM_VERTS;

fn sphere_wireframe_unit() -> Vec<[f32; 3]> {
    let mut verts = Vec::with_capacity(VERTS_PER_SPHERE as usize);
    let step = std::f32::consts::TAU / SPHERE_SEGMENTS as f32;
    for ring in 0..SPHERE_RINGS {
        for i in 0..SPHERE_SEGMENTS {
            let a1 = i as f32 * step;
            let a2 = (i + 1) as f32 * step;
            let (c1, s1) = a1.sin_cos();
            let (c2, s2) = a2.sin_cos();
            match ring {
                0 => {
                    verts.push([c1, s1, 0.0]);
                    verts.push([c2, s2, 0.0]);
                }
                1 => {
                    verts.push([c1, 0.0, s1]);
                    verts.push([c2, 0.0, s2]);
                }
                _ => {
                    verts.push([0.0, c1, s1]);
                    verts.push([0.0, c2, s2]);
                }
            }
        }
    }
    verts
}

const FRUSTUM_EDGES: [(usize, usize); 12] = [
    (0, 1), (1, 3), (3, 2), (2, 0),
    (4, 5), (5, 7), (7, 6), (6, 4),
    (0, 4), (1, 5), (2, 6), (3, 7),
];

// ---------------------------------------------------------------------------
// Shader
// ---------------------------------------------------------------------------

const SHADER_SRC: &str = r#"
struct CameraUniform {
    position: vec3<f32>,
    view_projection_matrix: mat4x4<f32>,
    perspective_view_projection_matrix: mat4x4<f32>,
    view_projection_transposed: mat4x4<f32>,
    perspective_projection_invert: mat4x4<f32>,
    global_gamma: f32,
};

@group(0) @binding(0) var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.clip_position = camera.perspective_view_projection_matrix
        * vec4<f32>(input.position, 1.0);
    output.color = input.color;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(input.color, 1.0);
}
"#;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// A bounding sphere to render as a wireframe overlay.
pub struct SphereInstance {
    pub center: Point3<f32>,
    pub radius: f32,
    pub color: [f32; 3],
}

// ---------------------------------------------------------------------------
// DebugRenderer — standalone GPU debug overlay
// ---------------------------------------------------------------------------

/// GPU debug renderer for bounding spheres and camera frustum wireframes.
pub struct DebugRenderer {
    pipeline: RenderPipeline,
    bind_group_layout: BindGroupLayout,
    vertex_buffer: Buffer,
    device: Device,
    unit_sphere: Vec<[f32; 3]>,
    enabled: bool,
    bind_group: Option<BindGroup>,
    last_camera_buffer_ptr: usize,
}

impl DebugRenderer {
    pub fn new(device: &Device, format: TextureFormat) -> Self {
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Debug Shader"),
            source: ShaderSource::Wgsl(SHADER_SRC.into()),
        });

        let bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Debug Camera BindGroup Layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Debug Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let vertex_buffer_layout = VertexBufferLayout {
            array_stride: 24,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    format: VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                VertexAttribute {
                    format: VertexFormat::Float32x3,
                    offset: 12,
                    shader_location: 1,
                },
            ],
        };

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Debug Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Some(vertex_buffer_layout)],
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::LineList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: Some(false),
                depth_compare: Some(CompareFunction::Always),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: MultisampleState::default(),
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(ColorTargetState {
                    format,
                    blend: Some(BlendState {
                        color: BlendComponent {
                            src_factor: BlendFactor::SrcAlpha,
                            dst_factor: BlendFactor::OneMinusSrcAlpha,
                            operation: BlendOperation::Add,
                        },
                        alpha: BlendComponent {
                            src_factor: BlendFactor::One,
                            dst_factor: BlendFactor::Zero,
                            operation: BlendOperation::Add,
                        },
                    }),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });

        let vertex_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Debug Vertex Buffer"),
            size: MAX_VERTS as u64 * 24,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            bind_group_layout,
            vertex_buffer,
            device: device.clone(),
            unit_sphere: sphere_wireframe_unit(),
            enabled: true,
            bind_group: None,
            last_camera_buffer_ptr: 0,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    /// Draw bounding-sphere wireframes and (optionally) the camera frustum.
    pub fn render(
        &mut self,
        render_pass: &mut RenderPass,
        camera_buffer: &Buffer,
        spheres: &[SphereInstance],
        frustum_corners: Option<&[Point3<f32>; 8]>,
        queue: &Queue,
    ) {
        if !self.enabled {
            return;
        }

        let mut verts: Vec<f32> = Vec::with_capacity(MAX_VERTS as usize * 6);

        let count = spheres.len().min(MAX_SPHERE_INSTANCES as usize);
        for inst in &spheres[..count] {
            for unit_pos in &self.unit_sphere {
                verts.push(inst.center.x + unit_pos[0] * inst.radius);
                verts.push(inst.center.y + unit_pos[1] * inst.radius);
                verts.push(inst.center.z + unit_pos[2] * inst.radius);
                verts.extend_from_slice(&inst.color);
            }
        }

        if let Some(corners) = frustum_corners {
            for &(i, j) in &FRUSTUM_EDGES {
                for idx in [i, j] {
                    let c = corners[idx];
                    verts.push(c.x);
                    verts.push(c.y);
                    verts.push(c.z);
                    verts.extend_from_slice(&[1.0, 1.0, 0.0]);
                }
            }
        }

        if verts.is_empty() {
            return;
        }

        let bytes = unsafe {
            std::slice::from_raw_parts(verts.as_ptr() as *const u8, verts.len() * 4)
        };
        queue.write_buffer(&self.vertex_buffer, 0, bytes);

        let cb_ptr = camera_buffer as *const Buffer as usize;
        if cb_ptr != self.last_camera_buffer_ptr {
            let bg = self.device.create_bind_group(&BindGroupDescriptor {
                label: Some("Debug Camera BindGroup"),
                layout: &self.bind_group_layout,
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }],
            });
            self.bind_group = Some(bg);
            self.last_camera_buffer_ptr = cb_ptr;
        }

        let num_verts = verts.len() as u32 / 6;
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, self.bind_group.as_ref().unwrap(), &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..num_verts, 0..1);
    }
}

/// Compute the 8 world-space corners of the view frustum from the combined
/// perspective‑view‑projection matrix.
pub fn frustum_corners_from_matrix(matrix: &Matrix4<f32>) -> [Point3<f32>; 8] {
    let inv = matrix.invert().unwrap_or(Matrix4::identity());
    let ndc = [
        Vector4::new(-1.0, -1.0, -1.0, 1.0),
        Vector4::new(1.0, -1.0, -1.0, 1.0),
        Vector4::new(-1.0, 1.0, -1.0, 1.0),
        Vector4::new(1.0, 1.0, -1.0, 1.0),
        Vector4::new(-1.0, -1.0, 1.0, 1.0),
        Vector4::new(1.0, -1.0, 1.0, 1.0),
        Vector4::new(-1.0, 1.0, 1.0, 1.0),
        Vector4::new(1.0, 1.0, 1.0, 1.0),
    ];
    let mut corners = [Point3::new(0.0, 0.0, 0.0); 8];
    for (i, n) in ndc.iter().enumerate() {
        let w = inv * n;
        corners[i] = Point3::new(w.x / w.w, w.y / w.w, w.z / w.w);
    }
    corners
}

// ---------------------------------------------------------------------------
// Integration into the module system
// ---------------------------------------------------------------------------

/// Toggle state — stores configured key, edge-detection state, and enabled flag.
pub struct DebugToggleState {
    pub key: winit::keyboard::KeyCode,
    pub was_pressed: bool,
    pub enabled: bool,
}

/// ECS system that toggles the debug overlay on keypress.
pub fn sys_debug_toggle(
    input: Res<InputSnapshot>,
    mut state: ResMut<DebugToggleState>,
) {
    let pressed = input
        .0
        .button_state_any(&orbital_app::input::InputButton::Keyboard(
            PhysicalKey::Code(state.key),
        ))
        .map(|(_, s)| s)
        .unwrap_or(false);
    if pressed && !state.was_pressed {
        state.enabled = !state.enabled;
    }
    state.was_pressed = pressed;
}

/// [`RenderOverlay`] implementation that draws bounding spheres and frustum.
struct DebugRenderOverlay {
    inner: DebugRenderer,
}

impl RenderOverlay for DebugRenderOverlay {
    fn render(&mut self, ctx: RenderOverlayContext) {
        let enabled = ctx
            .ecs
            .get_resource::<DebugToggleState>()
            .map(|s| s.enabled)
            .unwrap_or(false);
        if !enabled {
            return;
        }

        let spheres = collect_spheres(ctx.ecs);
        let corners = camera_frustum_corners(ctx.ecs);

        let mut enc =
            ctx.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Debug Overlay Encoder"),
                });
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Debug Overlay RenderPass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: ctx.target_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            self.inner.render(
                &mut pass,
                ctx.camera_buffer,
                &spheres,
                corners.as_ref(),
                ctx.queue,
            );
        }
        ctx.queue.submit(vec![enc.finish()]);
    }
}

/// Module that adds debug overlay rendering to an application.
///
/// ```ignore
/// App::new()
///     .add_module(DebugModule::new().with_keybind(KeyCode::F3))
///     .liftoff(...);
/// ```
pub struct DebugModule {
    toggle_key: Option<KeyCode>,
}

impl DebugModule {
    pub fn new() -> Self {
        Self { toggle_key: None }
    }

    /// Set the key that toggles the debug overlay.
    ///
    /// Defaults to `F3` when not called.
    pub fn with_keybind(mut self, key: KeyCode) -> Self {
        self.toggle_key = Some(key);
        self
    }
}

impl Default for DebugModule {
    fn default() -> Self {
        Self::new()
    }
}

impl Module for DebugModule {
    fn setup(
        &self,
        ecs: &mut World,
        device: &Device,
        _queue: &Queue,
    ) -> Vec<Box<dyn System>> {
        // Surface format — needed for pipeline creation.
        let format = ecs
            .get_resource::<orbital_ecs_bridge::SurfaceFormatResource>()
            .map(|f| f.0)
            .unwrap_or(TextureFormat::Bgra8UnormSrgb);

        let inner = DebugRenderer::new(device, format);
        let overlay = DebugRenderOverlay { inner };

        ecs.insert_resource(DebugToggleState {
            key: self.toggle_key.unwrap_or(KeyCode::F3),
            was_pressed: false,
            enabled: false,
        });
        ecs.insert_resource(RenderOverlayResource(std::sync::Mutex::new(Box::new(
            overlay,
        ))));

        vec![sys_debug_toggle.into_system()]
    }
}

// ---------------------------------------------------------------------------
// ECS data collection helpers
// ---------------------------------------------------------------------------

fn collect_spheres(ecs: &World) -> Vec<SphereInstance> {
    let mut spheres = Vec::new();

    let realizations = match ecs.get_component_store::<ModelRealization>() {
        Some(s) => s,
        None => return spheres,
    };
    let instances = match ecs.get_component_store::<ModelInstances>() {
        Some(s) => s,
        None => return spheres,
    };

    for &eid in realizations.dense.as_slice() {
        let Some(real_idx) = realizations.sparse[eid] else { continue };
        let Some(inst_idx) = instances.sparse[eid] else { continue };

        let model = &realizations.components[real_idx].0;
        let mesh = model.mesh();
        let Some(bsphere) = mesh.bounding_sphere() else { continue };

        let model_instances = &instances.components[inst_idx];
        for transform in model_instances.0.values() {
            let m = transform.to_matrix();
            let center_h = m * Vector4::new(bsphere.center.x, bsphere.center.y, bsphere.center.z, 1.0);
            let world_center = Point3::new(center_h.x, center_h.y, center_h.z);

            let max_scale = transform
                .scale
                .x
                .max(transform.scale.y)
                .max(transform.scale.z);
            let world_radius = bsphere.radius * max_scale;

            spheres.push(SphereInstance {
                center: world_center,
                radius: world_radius,
                color: [0.0, 1.0, 0.0],
            });
        }
    }

    spheres
}

fn camera_frustum_corners(ecs: &World) -> Option<[Point3<f32>; 8]> {
    let active = ecs.get_resource::<ActiveCamera>()?;
    let store = ecs.get_resource::<EcsCameraStore>()?;
    let arc_camera = store.get(active.0.index)?;
    let camera = arc_camera.read().ok()?;
    let matrix = camera.perspective_view_projection_matrix();
    Some(frustum_corners_from_matrix(matrix))
}

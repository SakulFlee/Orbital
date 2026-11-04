use cgmath::{InnerSpace, Matrix4, Point3, SquareMatrix, Vector3, Vector4};
use orbital_app::{LayerRenderer, Module, RenderLayer, RenderOverlayContext};
use orbital_ecs::{IntoSystem, Res, ResMut, System, World};
use orbital_ecs_bridge::{
    ActiveCamera, CameraStore, DebugSelectedEntity, InputSnapshot, LightDescriptor, ModelInstances,
    ModelRealization, Position, ShadowDebugData,
};
use orbital_instance::Instance;
use orbital_light::LightType;
use orbital_shadow::{SHADOW_TYPE_DIRECTIONAL_CASCADE, SHADOW_TYPE_POINT, SHADOW_TYPE_SPOT};
use orbital_vertex::Vertex;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, BlendComponent, BlendFactor, BlendOperation, BlendState,
    Buffer, BufferBindingType, BufferDescriptor, BufferUsages, ColorTargetState, ColorWrites,
    Device, Features, FragmentState, IndexFormat, MultisampleState, PipelineLayoutDescriptor,
    PrimitiveState, PrimitiveTopology, Queue, RenderPass, RenderPipeline, RenderPipelineDescriptor,
    ShaderModuleDescriptor, ShaderSource, ShaderStages, TextureFormat, VertexAttribute,
    VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
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
const MAX_EXTRA_VERTS: u32 = 16_384;
const MAX_VERTS: u32 = MAX_SPHERE_INSTANCES * VERTS_PER_SPHERE + FRUSTUM_VERTS + MAX_EXTRA_VERTS;

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
    (0, 1),
    (1, 3),
    (3, 2),
    (2, 0),
    (4, 5),
    (5, 7),
    (7, 6),
    (6, 4),
    (0, 4),
    (1, 5),
    (2, 6),
    (3, 7),
];

const BOX_EDGE_INDICES: [(usize, usize); 12] = [
    (0, 1),
    (1, 2),
    (2, 3),
    (3, 0),
    (4, 5),
    (5, 6),
    (6, 7),
    (7, 4),
    (0, 4),
    (1, 5),
    (2, 6),
    (3, 7),
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

const WIREFRAME_SHADER_SRC: &str = r#"
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
    @location(5) m0: vec4<f32>,
    @location(6) m1: vec4<f32>,
    @location(7) m2: vec4<f32>,
    @location(8) m3: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    let world = input.m0 * input.position.x
        + input.m1 * input.position.y
        + input.m2 * input.position.z
        + input.m3;
    var output: VertexOutput;
    output.clip_position = camera.perspective_view_projection_matrix * world;
    return output;
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(0.85, 0.9, 1.0, 1.0);
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
    wireframe_pipeline: Option<RenderPipeline>,
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

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
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
            depth_stencil: None,
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

        let wireframe_pipeline = if device.features().contains(Features::POLYGON_MODE_LINE) {
            let wireframe_shader = device.create_shader_module(ShaderModuleDescriptor {
                label: Some("Debug Wireframe Shader"),
                source: ShaderSource::Wgsl(WIREFRAME_SHADER_SRC.into()),
            });
            Some(device.create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("Debug Wireframe Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: VertexState {
                    module: &wireframe_shader,
                    entry_point: Some("vs_main"),
                    compilation_options: Default::default(),
                    buffers: &[
                        Some(Vertex::complex_vertex_buffer_layout_descriptor()),
                        Some(Instance::vertex_buffer_layout_descriptor()),
                    ],
                },
                primitive: PrimitiveState {
                    topology: PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Line,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: MultisampleState::default(),
                fragment: Some(FragmentState {
                    module: &wireframe_shader,
                    entry_point: Some("fs_main"),
                    compilation_options: Default::default(),
                    targets: &[Some(ColorTargetState {
                        format,
                        blend: Some(BlendState::ALPHA_BLENDING),
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                multiview_mask: None,
                cache: None,
            }))
        } else {
            None
        };

        Self {
            pipeline,
            wireframe_pipeline,
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

    /// Draw bounding-sphere wireframes, (optionally) the camera frustum,
    /// and optional extra lines (e.g. light visualisations).
    ///
    /// `frustum_color` is used when `frustum_corners` is `Some`.
    /// Default to `[1.0, 1.0, 0.0]` (yellow) for the live frustum,
    /// `[0.0, 1.0, 1.0]` (cyan) for the live frustum when a frozen
    /// frustum is also visible.
    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &mut self,
        render_pass: &mut RenderPass,
        camera_buffer: &Buffer,
        spheres: &[SphereInstance],
        frustum_corners: Option<&[Point3<f32>; 8]>,
        frustum_color: [f32; 3],
        extra_lines: &[[f32; 6]],
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
                    verts.extend_from_slice(&frustum_color);
                }
            }
        }

        // Extra lines (lights, bounds, grid, shadow frusta, selection).
        for line_vert in extra_lines.iter().take(MAX_EXTRA_VERTS as usize) {
            verts.push(line_vert[0]);
            verts.push(line_vert[1]);
            verts.push(line_vert[2]);
            verts.push(line_vert[3]);
            verts.push(line_vert[4]);
            verts.push(line_vert[5]);
        }

        if verts.is_empty() {
            return;
        }

        let bytes =
            unsafe { std::slice::from_raw_parts(verts.as_ptr() as *const u8, verts.len() * 4) };
        queue.write_buffer(&self.vertex_buffer, 0, bytes);

        self.ensure_camera_bind_group(camera_buffer);

        let num_verts = verts.len() as u32 / 6;
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, self.bind_group.as_ref().unwrap(), &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..num_verts, 0..1);
    }

    fn ensure_camera_bind_group(&mut self, camera_buffer: &Buffer) {
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
    }

    /// Draw every realized model's triangles as a wireframe.
    ///
    /// Uses the model's existing vertex, index, and instance buffers, so this
    /// does not duplicate mesh data. Backends without line polygon mode simply
    /// skip the pass.
    pub fn render_wireframe(
        &mut self,
        render_pass: &mut RenderPass,
        camera_buffer: &Buffer,
        ecs: &World,
    ) {
        if !self.enabled || self.wireframe_pipeline.is_none() {
            return;
        }

        self.ensure_camera_bind_group(camera_buffer);
        let Some(pipeline) = self.wireframe_pipeline.as_ref() else {
            return;
        };
        let Some(realizations) = ecs.get_component_store::<ModelRealization>() else {
            return;
        };

        render_pass.set_pipeline(pipeline);
        render_pass.set_bind_group(0, self.bind_group.as_ref().unwrap(), &[]);

        for realization in realizations.components.iter() {
            let model = &realization.0;
            let mesh = model.mesh();
            if mesh.index_count() == 0 || model.instance_count() == 0 {
                continue;
            }

            render_pass.set_vertex_buffer(0, mesh.vertex_buffer().slice(..));
            render_pass.set_vertex_buffer(1, model.instance_buffer().slice(..));
            render_pass.set_index_buffer(mesh.index_buffer().slice(..), IndexFormat::Uint32);
            render_pass.draw_indexed(0..mesh.index_count(), 0, 0..model.instance_count());
        }
    }
}

/// Compute the 8 world-space corners of the view frustum from the combined
/// perspective‑view‑projection matrix.
///
/// Uses the wgpu clip convention: the near plane is at z_ndc = 0.
pub fn frustum_corners_from_matrix(matrix: &Matrix4<f32>) -> [Point3<f32>; 8] {
    let inv = matrix.invert().unwrap_or(Matrix4::identity());
    let ndc = [
        Vector4::new(-1.0, -1.0, 0.0, 1.0),
        Vector4::new(1.0, -1.0, 0.0, 1.0),
        Vector4::new(-1.0, 1.0, 0.0, 1.0),
        Vector4::new(1.0, 1.0, 0.0, 1.0),
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

/// Which model bounds the debug overlay draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BoundsMode {
    Off,
    #[default]
    Spheres,
    Boxes,
    Both,
}

impl BoundsMode {
    pub fn spheres(self) -> bool {
        matches!(self, Self::Spheres | Self::Both)
    }

    pub fn boxes(self) -> bool {
        matches!(self, Self::Boxes | Self::Both)
    }
}

/// Per-category switches for the debug render overlay.
#[derive(Debug, Clone, Copy)]
pub struct DebugRenderSettings {
    /// Master switch, toggled by F3 by default.
    pub enabled: bool,
    pub bounds: BoundsMode,
    pub frustum: bool,
    pub lights: bool,
    pub wireframe: bool,
    pub grid_axes: bool,
    /// Shadow-map frusta from the previous shadow pass.
    pub shadow_frusta: bool,
    /// Highlight the entity selected in the ECS debug panel.
    pub selection: bool,
}

impl Default for DebugRenderSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            bounds: BoundsMode::Spheres,
            frustum: true,
            lights: true,
            wireframe: false,
            grid_axes: false,
            shadow_frusta: false,
            selection: false,
        }
    }
}

/// Toggle state — stores configured key, edge-detection state, and settings.
pub struct DebugToggleState {
    pub key: winit::keyboard::KeyCode,
    pub was_pressed: bool,
    pub settings: DebugRenderSettings,
}

/// ECS system that toggles the debug overlay on keypress.
pub fn sys_debug_toggle(input: Res<InputSnapshot>, mut state: ResMut<DebugToggleState>) {
    let pressed = input
        .0
        .button_state_any(&orbital_app::input::InputButton::Keyboard(
            PhysicalKey::Code(state.key),
        ))
        .map(|(_, s)| s)
        .unwrap_or(false);
    if pressed && !state.was_pressed {
        state.settings.enabled = !state.settings.enabled;
    }
    state.was_pressed = pressed;
}

/// [`LayerRenderer`] implementation that draws bounding spheres and frustum.
struct DebugRenderOverlay {
    inner: DebugRenderer,
}

impl LayerRenderer for DebugRenderOverlay {
    fn layer(&self) -> RenderLayer {
        RenderLayer::Debug
    }

    fn render(&mut self, ctx: RenderOverlayContext) {
        let settings = ctx
            .ecs
            .get_resource::<DebugToggleState>()
            .map(|state| state.settings)
            .unwrap_or_default();
        if !settings.enabled {
            return;
        }

        let frozen_data = ctx
            .ecs
            .get_resource::<orbital_ecs_bridge::FrozenFrustum>()
            .and_then(|f| f.0.clone());
        let frozen_frustum = frozen_data.as_ref().map(|f| &f.frustum);

        let spheres = if settings.bounds.spheres() {
            collect_spheres(ctx.ecs, frozen_frustum)
        } else {
            Vec::new()
        };

        let mut extra_lines = Vec::new();
        if settings.bounds.boxes() {
            extra_lines.extend(collect_box_lines(ctx.ecs, frozen_frustum));
        }
        if settings.lights {
            extra_lines.extend(collect_light_lines(ctx.ecs));
        }
        if settings.grid_axes {
            extra_lines.extend(grid_axes_lines(ctx.ecs));
        }
        if settings.shadow_frusta {
            extra_lines.extend(shadow_frustum_lines(ctx.ecs));
        }
        if settings.selection {
            extra_lines.extend(selection_lines(ctx.ecs));
        }

        let live_corners = settings
            .frustum
            .then(|| camera_frustum_corners(ctx.ecs))
            .flatten();
        let frozen_corners = settings.frustum.then(|| {
            frozen_data.as_ref().map(|frozen| {
                frustum_corners_from_matrix(&frozen.perspective_view_projection_matrix)
            })
        });

        let Some(enc) = ctx.encoder else {
            return;
        };
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

            if let Some(frozen_corners) = frozen_corners.flatten() {
                self.inner.render(
                    &mut pass,
                    ctx.camera_buffer,
                    &spheres,
                    Some(&frozen_corners),
                    [1.0, 1.0, 0.0],
                    &extra_lines,
                    ctx.queue,
                );
                if let Some(corners) = live_corners {
                    self.inner.render(
                        &mut pass,
                        ctx.camera_buffer,
                        &[],
                        Some(&corners),
                        [0.0, 1.0, 1.0],
                        &[],
                        ctx.queue,
                    );
                }
            } else {
                self.inner.render(
                    &mut pass,
                    ctx.camera_buffer,
                    &spheres,
                    live_corners.as_ref(),
                    [1.0, 1.0, 0.0],
                    &extra_lines,
                    ctx.queue,
                );
            }

            if settings.wireframe {
                self.inner
                    .render_wireframe(&mut pass, ctx.camera_buffer, ctx.ecs);
            }
        }
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
    freeze_key: Option<KeyCode>,
}

impl DebugModule {
    pub fn new() -> Self {
        Self {
            toggle_key: None,
            freeze_key: None,
        }
    }

    /// Set the key that toggles the debug overlay.
    ///
    /// Defaults to `F3` when not called.
    pub fn with_toggle_key(mut self, key: KeyCode) -> Self {
        self.toggle_key = Some(key);
        self
    }

    /// Set the key that freezes / unfreezes the culling frustum.
    ///
    /// Defaults to `F4` when not called.
    pub fn with_freeze_key(mut self, key: KeyCode) -> Self {
        self.freeze_key = Some(key);
        self
    }
}

impl Default for DebugModule {
    fn default() -> Self {
        Self::new()
    }
}

impl Module for DebugModule {
    fn setup(&self, ecs: &mut World, _device: &Device, _queue: &Queue) -> Vec<Box<dyn System>> {
        ecs.insert_resource(orbital_app::FreezeKeyConfig(
            self.freeze_key.unwrap_or(KeyCode::F4),
        ));
        ecs.insert_resource(DebugToggleState {
            key: self.toggle_key.unwrap_or(KeyCode::F3),
            was_pressed: false,
            settings: DebugRenderSettings::default(),
        });

        vec![sys_debug_toggle.into_system()]
    }

    fn register_overlays(
        &self,
        ecs: &mut World,
        layer_renderers: &mut Vec<Box<dyn LayerRenderer>>,
    ) {
        let format = ecs
            .get_resource::<orbital_ecs_bridge::SurfaceFormatResource>()
            .map(|f| f.0)
            .unwrap_or(TextureFormat::Bgra8UnormSrgb);

        // Get device from ECS resources.
        let device = match ecs.get_resource::<orbital_ecs_bridge::DeviceResource>() {
            Some(d) => d,
            None => {
                orbital_core::logging::warn!(
                    "DebugModule: no DeviceResource, skipping overlay registration"
                );
                return;
            }
        };

        let inner = DebugRenderer::new(&device.0, format);
        let overlay = DebugRenderOverlay { inner };
        layer_renderers.push(Box::new(overlay));
    }
}

// ---------------------------------------------------------------------------
// ECS data collection helpers
// ---------------------------------------------------------------------------

fn collect_spheres(
    ecs: &World,
    frozen_frustum: Option<&orbital_camera::Frustum>,
) -> Vec<SphereInstance> {
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
        let Some(real_idx) = realizations.sparse[eid] else {
            continue;
        };
        let Some(inst_idx) = instances.sparse[eid] else {
            continue;
        };

        let model = &realizations.components[real_idx].0;
        let mesh = model.mesh();
        let Some(bsphere) = mesh.bounding_sphere() else {
            continue;
        };

        let model_instances = &instances.components[inst_idx];
        for transform in model_instances.0.values() {
            let m = transform.to_matrix();
            let center_h =
                m * Vector4::new(bsphere.center.x, bsphere.center.y, bsphere.center.z, 1.0);
            let world_center = Point3::new(center_h.x, center_h.y, center_h.z);

            let max_scale = transform
                .scale
                .x
                .max(transform.scale.y)
                .max(transform.scale.z);
            let world_radius = bsphere.radius * max_scale;

            let color = match frozen_frustum {
                Some(frustum) => {
                    if frustum.intersects_sphere(&world_center, world_radius) {
                        [0.0, 1.0, 0.0] // green — visible
                    } else {
                        [1.0, 0.0, 0.0] // red — culled
                    }
                }
                None => [0.0, 1.0, 0.0],
            };

            spheres.push(SphereInstance {
                center: world_center,
                radius: world_radius,
                color,
            });
        }
    }

    spheres
}

fn collect_box_lines(
    ecs: &World,
    frozen_frustum: Option<&orbital_camera::Frustum>,
) -> Vec<[f32; 6]> {
    let mut lines = Vec::new();

    let Some(realizations) = ecs.get_component_store::<ModelRealization>() else {
        return lines;
    };
    let Some(instances) = ecs.get_component_store::<ModelInstances>() else {
        return lines;
    };

    for &eid in realizations.dense.as_slice() {
        let Some(real_idx) = realizations.sparse[eid] else {
            continue;
        };
        let Some(inst_idx) = instances.sparse[eid] else {
            continue;
        };

        let model = &realizations.components[real_idx].0;
        let Some(bounds) = model.mesh().bounding_box() else {
            continue;
        };

        let model_instances = &instances.components[inst_idx];
        for transform in model_instances.0.values() {
            let matrix = transform.to_matrix();
            let corners = bounds.corners();
            let world = corners.map(|corner| {
                let p = matrix * Vector4::new(corner.x, corner.y, corner.z, 1.0);
                Point3::new(p.x, p.y, p.z)
            });

            let local_center = Point3::new(
                (bounds.min.x + bounds.max.x) * 0.5,
                (bounds.min.y + bounds.max.y) * 0.5,
                (bounds.min.z + bounds.max.z) * 0.5,
            );
            let center = matrix * Vector4::new(local_center.x, local_center.y, local_center.z, 1.0);
            let radius = (bounds.max - bounds.min).magnitude() * 0.5;
            let max_scale = transform
                .scale
                .x
                .max(transform.scale.y)
                .max(transform.scale.z);
            let color = match frozen_frustum {
                Some(frustum) => {
                    let center = Point3::new(center.x, center.y, center.z);
                    if frustum.intersects_sphere(&center, radius * max_scale) {
                        [0.0, 1.0, 0.0]
                    } else {
                        [1.0, 0.0, 0.0]
                    }
                }
                None => [0.85, 0.25, 1.0],
            };

            for (a, b) in BOX_EDGE_INDICES {
                lines.push([
                    world[a].x, world[a].y, world[a].z, color[0], color[1], color[2],
                ]);
                lines.push([
                    world[b].x, world[b].y, world[b].z, color[0], color[1], color[2],
                ]);
            }
        }
    }

    lines
}

fn grid_axes_lines(ecs: &World) -> Vec<[f32; 6]> {
    let center = ecs
        .get_resource::<ActiveCamera>()
        .and_then(|active| {
            ecs.get_component_store::<Position>()
                .and_then(|positions| positions.get_component(active.0.index).copied())
        })
        .map(|position| (position.0.x, position.0.z))
        .unwrap_or((0.0, 0.0));

    // Follow the camera so the grid does not visibly end under normal
    // movement. Two-unit spacing keeps the line count bounded while still
    // reading as a useful world scale.
    const EXTENT: f32 = 200.0;
    const STEP: f32 = 2.0;
    let grid_color = [0.22, 0.25, 0.30];
    let axis_x = [0.95, 0.2, 0.2];
    let axis_y = [0.2, 0.95, 0.2];
    let axis_z = [0.2, 0.35, 1.0];

    let start_x = ((center.0 - EXTENT) / STEP).floor() * STEP;
    let end_x = center.0 + EXTENT;
    let start_z = ((center.1 - EXTENT) / STEP).floor() * STEP;
    let end_z = center.1 + EXTENT;

    let mut lines = Vec::new();
    let mut x = start_x;
    while x <= end_x {
        lines.push([x, 0.0, start_z, grid_color[0], grid_color[1], grid_color[2]]);
        lines.push([x, 0.0, end_z, grid_color[0], grid_color[1], grid_color[2]]);
        x += STEP;
    }

    let mut z = start_z;
    while z <= end_z {
        lines.push([start_x, 0.0, z, grid_color[0], grid_color[1], grid_color[2]]);
        lines.push([end_x, 0.0, z, grid_color[0], grid_color[1], grid_color[2]]);
        z += STEP;
    }

    // World axes stay at the origin; the grid itself follows the camera.
    lines.push([-EXTENT, 0.0, 0.0, axis_x[0], axis_x[1], axis_x[2]]);
    lines.push([EXTENT, 0.0, 0.0, axis_x[0], axis_x[1], axis_x[2]]);
    lines.push([0.0, 0.0, -EXTENT, axis_z[0], axis_z[1], axis_z[2]]);
    lines.push([0.0, 0.0, EXTENT, axis_z[0], axis_z[1], axis_z[2]]);
    lines.push([0.0, 0.0, 0.0, axis_y[0], axis_y[1], axis_y[2]]);
    lines.push([0.0, 5.0, 0.0, axis_y[0], axis_y[1], axis_y[2]]);

    lines
}

fn shadow_frustum_lines(ecs: &World) -> Vec<[f32; 6]> {
    let Some(data) = ecs.get_resource::<ShadowDebugData>() else {
        return Vec::new();
    };
    let mut lines = Vec::new();

    for slot in data
        .0
        .slots
        .iter()
        .take(data.0.active_slot_count() as usize)
    {
        let color = match slot.shadow_type {
            SHADOW_TYPE_DIRECTIONAL_CASCADE => [0.1, 0.6, 1.0],
            SHADOW_TYPE_SPOT => [1.0, 0.3, 0.1],
            SHADOW_TYPE_POINT => [1.0, 0.2, 0.8],
            _ => [0.8, 0.8, 0.8],
        };

        // Point shadows use a cube map; their slot matrix is only a
        // translation, so a frustum would be meaningless. Mark the light
        // position instead.
        if slot.shadow_type == SHADOW_TYPE_POINT {
            let position = Point3::new(
                slot.light_view_proj[3][0],
                slot.light_view_proj[3][1],
                slot.light_view_proj[3][2],
            );
            lines.extend(light_crosshair(position, color));
            continue;
        }

        let matrix = Matrix4::from(slot.light_view_proj);
        let corners = frustum_corners_from_matrix(&matrix);

        for (a, b) in FRUSTUM_EDGES {
            lines.push([
                corners[a].x,
                corners[a].y,
                corners[a].z,
                color[0],
                color[1],
                color[2],
            ]);
            lines.push([
                corners[b].x,
                corners[b].y,
                corners[b].z,
                color[0],
                color[1],
                color[2],
            ]);
        }
    }

    lines
}

fn selection_lines(ecs: &World) -> Vec<[f32; 6]> {
    let Some(selected) = ecs
        .get_resource::<DebugSelectedEntity>()
        .and_then(|selected| selected.0)
    else {
        return Vec::new();
    };
    let Some(realizations) = ecs.get_component_store::<ModelRealization>() else {
        return Vec::new();
    };
    let Some(instances) = ecs.get_component_store::<ModelInstances>() else {
        return Vec::new();
    };
    let Some(real_idx) = realizations.sparse.get(selected.index).copied().flatten() else {
        return Vec::new();
    };
    let Some(inst_idx) = instances.sparse.get(selected.index).copied().flatten() else {
        return Vec::new();
    };

    let model = &realizations.components[real_idx].0;
    let Some(bounds) = model.mesh().bounding_box() else {
        return Vec::new();
    };
    let color = [0.2, 1.0, 0.3];
    let mut lines = Vec::new();

    for transform in instances.components[inst_idx].0.values() {
        let matrix = transform.to_matrix();
        let world = bounds.corners().map(|corner| {
            let p = matrix * Vector4::new(corner.x, corner.y, corner.z, 1.0);
            Point3::new(p.x, p.y, p.z)
        });

        for (a, b) in BOX_EDGE_INDICES {
            lines.push([
                world[a].x, world[a].y, world[a].z, color[0], color[1], color[2],
            ]);
            lines.push([
                world[b].x, world[b].y, world[b].z, color[0], color[1], color[2],
            ]);
        }
    }

    lines
}

fn camera_frustum_corners(ecs: &World) -> Option<[Point3<f32>; 8]> {
    let active = ecs.get_resource::<ActiveCamera>()?;
    let store = ecs.get_resource::<CameraStore>()?;
    let arc_camera = store.get(active.0.index)?;
    let camera = arc_camera.read().ok()?;
    let matrix = camera.perspective_view_projection_matrix();
    Some(frustum_corners_from_matrix(matrix))
}

// ---------------------------------------------------------------------------
// Light debug visualisation
// ---------------------------------------------------------------------------

/// Generate a small crosshair at `position` (3 axis-aligned lines, 6 verts).
fn light_crosshair(position: Point3<f32>, color: [f32; 3]) -> Vec<[f32; 6]> {
    let s = 0.25;
    vec![
        [
            position.x - s,
            position.y,
            position.z,
            color[0],
            color[1],
            color[2],
        ],
        [
            position.x + s,
            position.y,
            position.z,
            color[0],
            color[1],
            color[2],
        ],
        [
            position.x,
            position.y - s,
            position.z,
            color[0],
            color[1],
            color[2],
        ],
        [
            position.x,
            position.y + s,
            position.z,
            color[0],
            color[1],
            color[2],
        ],
        [
            position.x,
            position.y,
            position.z - s,
            color[0],
            color[1],
            color[2],
        ],
        [
            position.x,
            position.y,
            position.z + s,
            color[0],
            color[1],
            color[2],
        ],
    ]
}

/// Generate a directional arrow: stem + crosshair at the tip.
fn light_arrow(position: Point3<f32>, direction: Vector3<f32>, color: [f32; 3]) -> Vec<[f32; 6]> {
    let dir = direction.normalize();
    let length = 2.0;
    let tip = Point3::new(
        position.x + dir.x * length,
        position.y + dir.y * length,
        position.z + dir.z * length,
    );
    let mut lines = Vec::with_capacity(8);
    // Stem
    lines.push([
        position.x, position.y, position.z, color[0], color[1], color[2],
    ]);
    lines.push([tip.x, tip.y, tip.z, color[0], color[1], color[2]]);
    // Arrowhead crosshair at tip
    lines.extend(light_crosshair(tip, color));
    lines
}

/// Generate a spot-light cone: center line + 8 apex-to-rim lines + rim ring.
fn light_cone(
    position: Point3<f32>,
    direction: Vector3<f32>,
    outer_angle: f32,
    color: [f32; 3],
) -> Vec<[f32; 6]> {
    let dir = direction.normalize();
    let range = 2.0;
    let half_angle = outer_angle.clamp(0.01, 1.5);

    let tip = Point3::new(
        position.x + dir.x * range,
        position.y + dir.y * range,
        position.z + dir.z * range,
    );

    let mut lines = Vec::with_capacity(34);
    // Center line
    lines.push([
        position.x, position.y, position.z, color[0], color[1], color[2],
    ]);
    lines.push([tip.x, tip.y, tip.z, color[0], color[1], color[2]]);

    // Perpendicular basis
    let up = if dir.y.abs() < 0.9 {
        Vector3::unit_y()
    } else {
        Vector3::unit_z()
    };
    let right = dir.cross(up).normalize();
    let up_actual = right.cross(dir).normalize();

    let radius = range * half_angle.tan();
    let segments = 8u32;
    let step = std::f32::consts::TAU / segments as f32;

    let mut rim = Vec::with_capacity(segments as usize);
    for i in 0..segments {
        let theta = i as f32 * step;
        let (s, c) = theta.sin_cos();
        let perp = right * c + up_actual * s;
        let p = Point3::new(
            position.x + dir.x * range + perp.x * radius,
            position.y + dir.y * range + perp.y * radius,
            position.z + dir.z * range + perp.z * radius,
        );
        lines.push([
            position.x, position.y, position.z, color[0], color[1], color[2],
        ]);
        lines.push([p.x, p.y, p.z, color[0], color[1], color[2]]);
        rim.push(p);
    }
    // Rim ring
    for i in 0..segments as usize {
        let j = (i + 1) % segments as usize;
        lines.push([rim[i].x, rim[i].y, rim[i].z, color[0], color[1], color[2]]);
        lines.push([rim[j].x, rim[j].y, rim[j].z, color[0], color[1], color[2]]);
    }

    lines
}

/// Read lights from ECS and return corresponding line vertex data.
fn collect_light_lines(ecs: &World) -> Vec<[f32; 6]> {
    let descs = match ecs.get_component_store::<LightDescriptor>() {
        Some(s) => s,
        None => return Vec::new(),
    };
    let positions = match ecs.get_component_store::<Position>() {
        Some(s) => s,
        None => return Vec::new(),
    };

    let mut lines = Vec::new();
    for &eid in descs.dense.as_slice() {
        let desc_idx = match descs.sparse.get(eid).copied().flatten() {
            Some(i) => i,
            None => continue,
        };
        let pos_idx = match positions.sparse.get(eid).copied().flatten() {
            Some(i) => i,
            None => continue,
        };

        let desc = &descs.components[desc_idx];
        let pos = &positions.components[pos_idx];
        let color = [desc.color.x, desc.color.y, desc.color.z];

        match desc.light_type {
            LightType::Point { .. } => {
                lines.extend(light_crosshair(
                    Point3::new(pos.0.x, pos.0.y, pos.0.z),
                    color,
                ));
            }
            LightType::Directional { .. } => {
                lines.extend(light_arrow(
                    Point3::new(pos.0.x, pos.0.y, pos.0.z),
                    -desc.direction, // incidence direction: arrow points where light comes FROM
                    color,
                ));
            }
            LightType::Spot {
                outer_cone_angle, ..
            } => {
                lines.extend(light_cone(
                    Point3::new(pos.0.x, pos.0.y, pos.0.z),
                    desc.direction,
                    outer_cone_angle,
                    color,
                ));
            }
        }
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounds_mode_selects_each_shape_independently() {
        assert!(!BoundsMode::Off.spheres());
        assert!(!BoundsMode::Off.boxes());
        assert!(BoundsMode::Spheres.spheres());
        assert!(!BoundsMode::Spheres.boxes());
        assert!(!BoundsMode::Boxes.spheres());
        assert!(BoundsMode::Boxes.boxes());
        assert!(BoundsMode::Both.spheres());
        assert!(BoundsMode::Both.boxes());
    }

    #[test]
    fn debug_overlay_defaults_preserve_existing_behavior() {
        let settings = DebugRenderSettings::default();
        assert!(!settings.enabled);
        assert_eq!(settings.bounds, BoundsMode::Spheres);
        assert!(settings.frustum);
        assert!(settings.lights);
        assert!(!settings.wireframe);
        assert!(!settings.grid_axes);
        assert!(!settings.shadow_frusta);
        assert!(!settings.selection);
    }
}

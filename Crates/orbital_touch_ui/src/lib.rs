//! On-screen touch controls for the Orbital engine.
//!
//! Provides a [`RenderOverlay`] that draws a virtual joystick in screen space
//! whenever the engine's default touch scheme has an active movement finger
//! (see [`orbital_input::InputState::touch_gesture`]), plus a [`TouchUiModule`]
//! to register it in an application.

use cgmath::{InnerSpace, Vector2};
use orbital_app::{
    touch_controls, Module, RenderOverlay, RenderOverlayContext, RenderOverlayResource,
};
use orbital_ecs::{System, World};
use orbital_ecs_bridge::{InputSnapshot, SurfaceFormatResource};
use wgpu::{
    BlendComponent, BlendFactor, BlendOperation, BlendState, Buffer, BufferDescriptor,
    BufferUsages, ColorTargetState, ColorWrites, CommandEncoderDescriptor, Device, FragmentState,
    MultisampleState, PipelineLayoutDescriptor, PrimitiveState, PrimitiveTopology, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    ShaderModuleDescriptor, ShaderSource, TextureFormat, VertexAttribute, VertexBufferLayout,
    VertexFormat, VertexState, VertexStepMode,
};

// ---------------------------------------------------------------------------
// Shader
// ---------------------------------------------------------------------------

/// Screen-space 2D shader — no camera bindings, vertices arrive in NDC.
const SHADER_SRC: &str = r#"
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.clip_position = vec4<f32>(input.position, 0.0, 1.0);
    output.color = input.color;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}
"#;

const MAX_VERTS: u32 = 512;
const VERTEX_STRIDE: u64 = 24;
const CIRCLE_SEGMENTS: u32 = 32;
const KNOB_SEGMENTS: u32 = 24;
const KNOB_RADIUS_RATIO: f64 = 0.45;
const HINT_SEGMENTS: u32 = 24;

/// Radius (window pixels) of the idle "move"/"look" hint circles.
const HINT_RADIUS: f64 = 45.0;
/// Radius (window pixels) of the small knob dot inside the move hint.
const HINT_KNOB_RADIUS: f64 = 12.0;

/// Semi-transparent white for the joystick base ring.
const BASE_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 0.18];
/// Brighter white for the draggable knob.
const KNOB_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 0.6];
/// Base ring color at full joystick deflection (max-deflection feedback).
const BASE_COLOR_MAX: [f32; 4] = [1.0, 1.0, 1.0, 0.32];
/// Knob color at full joystick deflection.
const KNOB_COLOR_MAX: [f32; 4] = [1.0, 1.0, 1.0, 0.8];
/// Knob radius (as a fraction of the base radius) at full deflection.
const KNOB_RADIUS_MAX_RATIO: f64 = 0.55;

/// Faint white for the idle move-hint circle.
const HINT_MOVE_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 0.08];
/// Slightly brighter dot in the middle of the move hint.
const HINT_MOVE_KNOB_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 0.14];
/// Faint white for the idle look-hint circle.
const HINT_LOOK_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 0.08];

// ---------------------------------------------------------------------------
// Overlay
// ---------------------------------------------------------------------------

/// Draws the on-screen virtual joystick for the default touch control scheme.
///
/// Reads the current [`InputSnapshot`] resource and draws a base ring at the
/// movement finger's press point plus a knob at its current position. Nothing
/// is drawn while no movement finger is active.
pub struct JoystickOverlay {
    pipeline: RenderPipeline,
    vertex_buffer: Buffer,
}

impl JoystickOverlay {
    pub fn new(device: &Device, format: TextureFormat) -> Self {
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Touch UI Shader"),
            source: ShaderSource::Wgsl(SHADER_SRC.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Touch UI Pipeline Layout"),
            bind_group_layouts: &[],
            immediate_size: 0,
        });

        let vertex_buffer_layout = VertexBufferLayout {
            array_stride: VERTEX_STRIDE,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    format: VertexFormat::Float32x2,
                    offset: 0,
                    shader_location: 0,
                },
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: 8,
                    shader_location: 1,
                },
            ],
        };

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Touch UI Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Some(vertex_buffer_layout)],
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
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
            label: Some("Touch UI Vertex Buffer"),
            size: MAX_VERTS as u64 * VERTEX_STRIDE,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            vertex_buffer,
        }
    }
}

impl RenderOverlay for JoystickOverlay {
    fn render(&mut self, ctx: RenderOverlayContext) {
        let Some(input) = ctx.ecs.get_resource::<InputSnapshot>() else {
            return;
        };
        let Some(size) = input.0.surface_size() else {
            return;
        };
        let width = size.x as f64;
        let height = size.y as f64;

        let cfg = touch_controls();
        let gesture = input
            .0
            .touch_gesture_with_config(cfg.joystick_radius, cfg.joystick_deadzone);

        let mut verts = Vec::with_capacity(MAX_VERTS as usize * 6);

        if let Some(origin) = gesture.joystick_origin {
            // Active movement joystick — draw base + knob, with max-deflection
            // feedback (brighter base, larger knob) at full radius.
            let position = gesture.joystick_position.unwrap_or(origin);
            let at_max = (position - origin).magnitude() >= cfg.joystick_radius;
            let (base_color, knob_color, knob_ratio) = if at_max {
                (BASE_COLOR_MAX, KNOB_COLOR_MAX, KNOB_RADIUS_MAX_RATIO)
            } else {
                (BASE_COLOR, KNOB_COLOR, KNOB_RADIUS_RATIO)
            };

            push_circle(
                &mut verts,
                origin,
                cfg.joystick_radius,
                CIRCLE_SEGMENTS,
                base_color,
                width,
                height,
            );
            push_circle(
                &mut verts,
                position,
                cfg.joystick_radius * knob_ratio,
                KNOB_SEGMENTS,
                knob_color,
                width,
                height,
            );
        } else if cfg.show_idle_hints && input.0.has_seen_touch() && !gesture.active {
            // Touch device with nothing touching — hint at the two control zones.
            let hint_y = height * 0.5;
            let move_hint = Vector2::new(width * 0.2, hint_y);
            let look_hint = Vector2::new(width * 0.8, hint_y);

            push_circle(
                &mut verts,
                move_hint,
                HINT_RADIUS,
                HINT_SEGMENTS,
                HINT_MOVE_COLOR,
                width,
                height,
            );
            push_circle(
                &mut verts,
                move_hint,
                HINT_KNOB_RADIUS,
                HINT_SEGMENTS,
                HINT_MOVE_KNOB_COLOR,
                width,
                height,
            );
            push_circle(
                &mut verts,
                look_hint,
                HINT_RADIUS,
                HINT_SEGMENTS,
                HINT_LOOK_COLOR,
                width,
                height,
            );
        }

        if verts.is_empty() {
            return;
        }

        let bytes =
            unsafe { std::slice::from_raw_parts(verts.as_ptr() as *const u8, verts.len() * 4) };
        ctx.queue.write_buffer(&self.vertex_buffer, 0, bytes);
        let num_verts = verts.len() as u32 / 6;

        let mut enc = ctx
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Touch UI Encoder"),
            });
        {
            let mut pass = enc.begin_render_pass(&RenderPassDescriptor {
                label: Some("Touch UI RenderPass"),
                color_attachments: &[Some(RenderPassColorAttachment {
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

            pass.set_pipeline(&self.pipeline);
            pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            pass.draw(0..num_verts, 0..1);
        }
        ctx.queue.submit(vec![enc.finish()]);
    }
}

/// Append a filled circle (explicit triangle list) in NDC derived from pixel coordinates.
///
/// Emits `segments` triangles `(center, p_i, p_{i+1})` matching the pipeline's
/// `TriangleList` topology. A triangle-fan vertex stream would be misinterpreted
/// as disjoint triangles, producing a dashed/spiky circle.
fn push_circle(
    verts: &mut Vec<f32>,
    center: Vector2<f64>,
    radius: f64,
    segments: u32,
    color: [f32; 4],
    width: f64,
    height: f64,
) {
    let cx = (center.x / width) * 2.0 - 1.0;
    let cy = 1.0 - (center.y / height) * 2.0;
    let radius_x = radius / width * 2.0;
    let radius_y = radius / height * 2.0;

    let center_xy = [cx as f32, cy as f32];
    for i in 0..segments {
        let theta_a = i as f64 * std::f64::consts::TAU / segments as f64;
        let theta_b = (i + 1) as f64 * std::f64::consts::TAU / segments as f64;
        let (sa, ca) = theta_a.sin_cos();
        let (sb, cb) = theta_b.sin_cos();
        let pa = [cx + sa * radius_x, cy - ca * radius_y];
        let pb = [cx + sb * radius_x, cy - cb * radius_y];

        verts.extend_from_slice(&[
            center_xy[0],
            center_xy[1],
            color[0],
            color[1],
            color[2],
            color[3],
        ]);
        verts.extend_from_slice(&[
            pa[0] as f32,
            pa[1] as f32,
            color[0],
            color[1],
            color[2],
            color[3],
        ]);
        verts.extend_from_slice(&[
            pb[0] as f32,
            pb[1] as f32,
            color[0],
            color[1],
            color[2],
            color[3],
        ]);
    }
}

// ---------------------------------------------------------------------------
// Module
// ---------------------------------------------------------------------------

/// Module that registers the on-screen virtual joystick overlay.
///
/// ```ignore
/// App::new()
///     .add_module(TouchUiModule)
///     .liftoff(...);
/// ```
///
/// Safe to add alongside other overlay modules (e.g. `DebugModule`) — overlays
/// are composable and drawn in insertion order.
pub struct TouchUiModule;

impl Module for TouchUiModule {
    fn setup(&self, ecs: &mut World, device: &Device, _queue: &Queue) -> Vec<Box<dyn System>> {
        let format = ecs
            .get_resource::<SurfaceFormatResource>()
            .map(|f| f.0)
            .unwrap_or(TextureFormat::Bgra8UnormSrgb);

        let overlay = JoystickOverlay::new(device, format);

        if ecs.get_resource::<RenderOverlayResource>().is_none() {
            ecs.insert_resource(RenderOverlayResource::new());
        }
        if let Some(res) = ecs.get_resource_mut::<RenderOverlayResource>() {
            res.add(Box::new(overlay));
        }

        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_circle_emits_valid_triangle_list() {
        let mut verts = Vec::new();
        push_circle(
            &mut verts,
            Vector2::new(200.0, 300.0),
            70.0,
            32,
            [1.0, 1.0, 1.0, 1.0],
            1000.0,
            800.0,
        );

        // 32 triangles * 3 vertices * 6 floats.
        assert_eq!(verts.len(), 32 * 3 * 6);
        assert!(verts.iter().all(|v| v.is_finite()));

        // Every triangle shares the circle center as its first vertex.
        let cx = ((200.0 / 1000.0) * 2.0 - 1.0) as f32;
        let cy = (1.0 - (300.0 / 800.0) * 2.0) as f32;
        for triangle in verts.chunks_exact(6 * 3) {
            assert_eq!(triangle[0], cx);
            assert_eq!(triangle[1], cy);
        }
    }
}

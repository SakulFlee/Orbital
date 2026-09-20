use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferDescriptor, BufferUsages, ColorTargetState,
    ColorWrites, Device, FragmentState, MultisampleState, PipelineLayoutDescriptor, PrimitiveState,
    PrimitiveTopology, RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor,
    ShaderSource, VertexAttribute, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
};

/// GPU renderer for solid-color 2D shapes.
///
/// Renders 2D geometry (quads, triangles, circles) with per-vertex colors.
/// Uses an orthographic projection for screen-space or world-space 2D rendering.
pub struct Renderer2D {
    pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    bind_group_layout: BindGroupLayout,
}

/// Uniform buffer data for the 2D camera.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Camera2DUniform {
    /// Orthographic view-projection matrix.
    pub view_proj: [[f32; 4]; 4],
    /// Screen size in pixels (width, height).
    pub screen_size: [f32; 2],
    /// Padding for alignment.
    pub _padding: [f32; 2],
}

/// WGSL shader for solid-color 2D rendering.
const SHADER_2D_SRC: &str = r#"
struct CameraUniform {
    view_proj: mat4x4<f32>,
    screen_size: vec2<f32>,
};

@group(0) @binding(0) var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) texcoord: vec2<f32>,
    @location(3) shape_params: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.clip_position = camera.view_proj * vec4<f32>(input.position, 0.0, 1.0);
    output.color = input.color;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}
"#;

impl Renderer2D {
    /// Creates a new 2D renderer with the given surface format.
    pub fn new(device: &Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("2D Shader"),
            source: ShaderSource::Wgsl(SHADER_2D_SRC.into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("2D Bind Group Layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("2D Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        // Vertex buffer layout matching orbital_2d::Vertex2D
        let vertex_buffer_layout = VertexBufferLayout {
            array_stride: 40, // 2 (pos) + 4 (color) + 2 (uv) + 2 (params) = 10 floats * 4 bytes
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    format: VertexFormat::Float32x2,
                    offset: 0,
                    shader_location: 0, // position
                },
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: 8,
                    shader_location: 1, // color
                },
                VertexAttribute {
                    format: VertexFormat::Float32x2,
                    offset: 24,
                    shader_location: 2, // texcoord
                },
                VertexAttribute {
                    format: VertexFormat::Float32x2,
                    offset: 32,
                    shader_location: 3, // shape_params
                },
            ],
        };

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("2D Pipeline"),
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
                cull_mode: None, // No culling for 2D
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None, // No depth testing for 2D
            multisample: MultisampleState::default(),
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::Zero,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });

        let vertex_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("2D Vertex Buffer"),
            size: 64 * 1024, // 64KB, enough for ~1600 vertices
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            vertex_buffer,
            bind_group_layout,
        }
    }

    /// Creates a bind group for the camera uniform.
    pub fn create_bind_group(&self, device: &Device, uniform_buffer: &Buffer) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            label: Some("2D Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        })
    }

    /// Returns a reference to the render pipeline.
    pub fn pipeline(&self) -> &RenderPipeline {
        &self.pipeline
    }

    /// Returns a reference to the vertex buffer.
    pub fn vertex_buffer(&self) -> &Buffer {
        &self.vertex_buffer
    }
}

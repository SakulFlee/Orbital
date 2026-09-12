use wgpu::{
    Buffer, BufferDescriptor, BufferUsages,
    ColorTargetState, ColorWrites, Device, FragmentState, MultisampleState, PipelineLayoutDescriptor,
    PrimitiveState, PrimitiveTopology, RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor,
    ShaderSource, VertexAttribute, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
};

/// GPU renderer for UI backgrounds using SDF rounded rectangles.
///
/// Renders UI elements with rounded corners using signed distance fields.
/// Works in screen-space with pixel coordinates converted to NDC.
pub struct UiRenderer {
    pipeline: RenderPipeline,
    vertex_buffer: Buffer,
}

/// WGSL shader for SDF rounded rectangle rendering.
const SHADER_UI_SRC: &str = r#"
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) texcoord: vec2<f32>,
    @location(3) shape_params: vec2<f32>,  // x = corner_radius, y = reserved
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) corner_radius: f32,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    // Positions already in NDC (CPU converts pixel -> NDC)
    output.clip_position = vec4<f32>(input.position, 0.0, 1.0);
    output.color = input.color;
    output.uv = input.texcoord;
    output.corner_radius = input.shape_params.x;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // SDF rounded rectangle
    let corner_radius = input.corner_radius;

    // Convert UV from [0,1] to centered [-1,1]
    let p = abs(input.uv * 2.0 - 1.0);

    // Distance from rounded rect edge
    // For a unit quad, we scale the corner radius by the quad size
    let d = length(max(p - vec2(1.0 - corner_radius), vec2(0.0))) - corner_radius;

    // Anti-aliased edge
    let aa_width = fwidth(d); // Screen-space derivatives for anti-aliasing
    let alpha = 1.0 - smoothstep(-aa_width, aa_width, d);

    return vec4<f32>(input.color.rgb, input.color.a * alpha);
}
"#;

impl UiRenderer {
    /// Creates a new UI renderer with the given surface format.
    pub fn new(device: &Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("UI Shader"),
            source: ShaderSource::Wgsl(SHADER_UI_SRC.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("UI Pipeline Layout"),
            bind_group_layouts: &[],
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
            label: Some("UI Pipeline"),
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
                cull_mode: None, // No culling for UI
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None, // No depth testing for UI
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
            label: Some("UI Vertex Buffer"),
            size: 64 * 1024, // 64KB, enough for ~1600 vertices
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            vertex_buffer,
        }
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

use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferDescriptor, BufferUsages, ColorTargetState,
    ColorWrites, Device, FragmentState, MultisampleState, PipelineLayoutDescriptor, PrimitiveState,
    PrimitiveTopology, Queue, RenderPipeline, RenderPipelineDescriptor, SamplerDescriptor,
    ShaderModuleDescriptor, ShaderSource, TextureAspect, TextureDescriptor, TextureDimension,
    TextureFormat, TextureSampleType, TextureViewDescriptor, TextureViewDimension, VertexAttribute,
    VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
};

use orbital_text::SdfAtlas;

/// GPU renderer for SDF text.
///
/// Renders text using signed distance field textures for crisp rendering at any scale.
pub struct TextRenderer {
    pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    bind_group_layout: BindGroupLayout,
    atlas_texture: Option<wgpu::Texture>,
    atlas_view: Option<wgpu::TextureView>,
    atlas_sampler: Option<wgpu::Sampler>,
}

/// Uniform buffer data for SDF text parameters.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct SdfParamsUniform {
    /// Smoothing factor for anti-aliasing.
    pub smoothing: f32,
    /// Width of outline (0 = no outline).
    pub outline_width: f32,
    /// Padding for vec4 alignment (WGSL requires 16-byte alignment for vec4).
    pub _padding: [f32; 2],
    /// Outline color (RGBA).
    pub outline_color: [f32; 4],
}

/// WGSL shader for SDF text rendering.
const SHADER_TEXT_SRC: &str = r#"
struct SdfParams {
    smoothing: f32,
    outline_width: f32,
    outline_color: vec4<f32>,
};

@group(1) @binding(0) var glyph_atlas: texture_2d<f32>;
@group(1) @binding(1) var glyph_sampler: sampler;
@group(2) @binding(0) var<uniform> sdf_params: SdfParams;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) texcoord: vec2<f32>,
    @location(3) shape_params: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) texcoord: vec2<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    // Positions already in NDC (CPU converts pixel -> NDC)
    output.clip_position = vec4<f32>(input.position, 0.0, 1.0);
    output.color = input.color;
    output.texcoord = input.texcoord;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let sdf = textureSample(glyph_atlas, glyph_sampler, input.texcoord).r;

    // Smoothstep for anti-aliased edges
    let alpha = smoothstep(
        0.5 - sdf_params.smoothing,
        0.5 + sdf_params.smoothing,
        sdf
    );

    // Optional outline
    if (sdf_params.outline_width > 0.0) {
        let outline_alpha = smoothstep(
            0.5 - sdf_params.outline_width - sdf_params.smoothing,
            0.5 - sdf_params.outline_width + sdf_params.smoothing,
            sdf
        );
        let outline = mix(sdf_params.outline_color.rgb, input.color.rgb, alpha);
        return vec4<f32>(outline, mix(sdf_params.outline_color.a, input.color.a, alpha));
    }

    return vec4<f32>(input.color.rgb, input.color.a * alpha);
}
"#;

impl TextRenderer {
    /// Creates a new text renderer with the given surface format.
    pub fn new(device: &Device, format: TextureFormat) -> Self {
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Text Shader"),
            source: ShaderSource::Wgsl(SHADER_TEXT_SRC.into()),
        });

        // Bind group layout for glyph atlas texture + sampler
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Text Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        // Bind group layout for SDF params uniform
        let params_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("SDF Params Bind Group Layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Text Pipeline Layout"),
            bind_group_layouts: &[
                None, // No camera bind group for screen-space text
                Some(&bind_group_layout),
                Some(&params_bind_group_layout),
            ],
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
            label: Some("Text Pipeline"),
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
                cull_mode: None, // No culling for text
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None, // No depth testing for text
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
            label: Some("Text Vertex Buffer"),
            size: 256 * 1024, // 256KB, enough for ~6400 vertices (text is more dense)
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            vertex_buffer,
            bind_group_layout,
            atlas_texture: None,
            atlas_view: None,
            atlas_sampler: None,
        }
    }

    /// Uploads an SDF atlas to the GPU.
    pub fn upload_atlas(&mut self, device: &Device, queue: &Queue, atlas: &SdfAtlas) {
        // Create texture
        let texture = device.create_texture(&TextureDescriptor {
            label: Some("SDF Atlas Texture"),
            size: wgpu::Extent3d {
                width: atlas.width,
                height: atlas.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        // Upload atlas data
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            atlas.data(),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * atlas.width),
                rows_per_image: Some(atlas.height),
            },
            wgpu::Extent3d {
                width: atlas.width,
                height: atlas.height,
                depth_or_array_layers: 1,
            },
        );

        let view = texture.create_view(&TextureViewDescriptor::default());

        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("SDF Atlas Sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        self.atlas_texture = Some(texture);
        self.atlas_view = Some(view);
        self.atlas_sampler = Some(sampler);
    }

    /// Creates a bind group for the atlas texture.
    pub fn create_atlas_bind_group(&self, device: &Device) -> Option<BindGroup> {
        let (view, sampler) = match (&self.atlas_view, &self.atlas_sampler) {
            (Some(v), Some(s)) => (v, s),
            _ => return None,
        };

        Some(device.create_bind_group(&BindGroupDescriptor {
            label: Some("Text Atlas Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        }))
    }

    /// Returns a reference to the render pipeline.
    pub fn pipeline(&self) -> &RenderPipeline {
        &self.pipeline
    }

    /// Returns a reference to the vertex buffer.
    pub fn vertex_buffer(&self) -> &Buffer {
        &self.vertex_buffer
    }

    /// Returns true if an atlas has been uploaded.
    pub fn has_atlas(&self) -> bool {
        self.atlas_texture.is_some()
    }
}

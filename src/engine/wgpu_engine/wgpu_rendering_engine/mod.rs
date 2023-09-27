use logical_device::LogicalDevice;
use wgpu::{
    include_wgsl, Adapter, BlendState, ColorTargetState, ColorWrites, CompareFunction,
    DepthBiasState, DepthStencilState, Device, Extent3d, Face, FragmentState, FrontFace, Instance,
    MultisampleState, PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology,
    RenderPipeline, RenderPipelineDescriptor, ShaderModule, StencilState, SurfaceConfiguration,
    TextureFormat, VertexState,
};
use winit::window::Window;

use crate::engine::{
    logical_device, DepthTexture, EngineResult, StandardAmbientLight, StandardInstance,
    StandardMaterial, StandardPointLight, TAmbientLight, TComputingEngine, TInstance, TMaterial,
    TPointLight, TRenderingEngine, TVertex, VertexPoint,
};

use super::wgpu_computing_engine::WGPUComputingEngine;

mod surface;
pub use surface::*;

mod camera;
pub use camera::*;

pub struct WGPURenderingEngine {
    computing_engine: WGPUComputingEngine,
    surface: Surface,
    render_pipeline: RenderPipeline,
    depth_texture: DepthTexture,
}

impl WGPURenderingEngine {
    pub fn new(window: &Window) -> EngineResult<Self> {
        let (computing_engine, surface) = Surface::from_window(window)?;

        let render_pipeline = Self::make_render_pipeline(
            computing_engine.logical_device(),
            surface.surface_texture_format(),
        )?;

        let depth_texture = DepthTexture::from_empty(
            computing_engine.logical_device(),
            Extent3d {
                width: window.inner_size().width,
                height: window.inner_size().height,
                depth_or_array_layers: 1,
            },
            DepthTexture::TEXTURE_FORMAT,
            &DepthTexture::SAMPLER_DESCRIPTOR,
            Some("Depth Texture"),
        )?;

        Ok(Self {
            computing_engine,
            surface,
            render_pipeline,
            depth_texture,
        })
    }

    fn make_shader(device: &Device) -> ShaderModule {
        device.create_shader_module(include_wgsl!("../../shaders/new_engine.wgsl"))
    }

    fn make_render_pipeline(
        logical_device: &LogicalDevice,
        surface_texture_format: TextureFormat,
    ) -> EngineResult<RenderPipeline> {
        let main_shader = Self::make_shader(logical_device.device());

        let render_pipeline_layout =
            logical_device
                .device()
                .create_pipeline_layout(&PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[
                        &StandardMaterial::bind_group_layout(logical_device),
                        &Camera::bind_group_layout(logical_device),
                        &StandardAmbientLight::bind_group_layout(logical_device),
                        &StandardPointLight::bind_group_layout(logical_device),
                        &StandardPointLight::bind_group_layout(logical_device),
                        &StandardPointLight::bind_group_layout(logical_device),
                        &StandardPointLight::bind_group_layout(logical_device),
                    ],
                    push_constant_ranges: &[],
                });

        Ok(logical_device
            .device()
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                // Vertex shader
                vertex: VertexState {
                    module: &main_shader,
                    entry_point: "vs_main",
                    // Vertex buffers
                    buffers: &[
                        VertexPoint::descriptor::<VertexPoint>(),
                        StandardInstance::descriptor(),
                    ],
                },
                // Fragment shader
                fragment: Some(FragmentState {
                    module: &main_shader,
                    entry_point: "fs_main",
                    // Store the resulting colours in a format
                    // that is equal to the surface format
                    targets: &[Some(ColorTargetState {
                        // Match the surface format
                        format: surface_texture_format,
                        // Replace pixels
                        blend: Some(BlendState::REPLACE),
                        // Use all colour channels
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                // How to interpret the vertices
                primitive: PrimitiveState {
                    // Every three vertices form a triangle
                    topology: PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    // A given triangle is is facing "forward" if it's arranged counter-clockwise
                    front_face: FrontFace::Ccw,
                    // Cull the triangle if it's the backside
                    cull_mode: Some(Face::Back),
                    // Fill the triangle
                    // Note: requires Features::NON_FILL_POLYGON_MODE if not Fill
                    polygon_mode: PolygonMode::Fill,
                    // Note: requires Features::DEPTH_CLIP_CONTROL
                    unclipped_depth: false,
                    // Note: requires Features::CONSERVATIVE_RASTERIZATION
                    conservative: false,
                },
                depth_stencil: Some(DepthStencilState {
                    format: DepthTexture::TEXTURE_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: CompareFunction::Less,
                    stencil: StencilState::default(),
                    bias: DepthBiasState::default(),
                }),
                multisample: MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            }))
    }
}

impl TComputingEngine for WGPURenderingEngine {
    fn instance(&self) -> &Instance {
        self.computing_engine.instance()
    }

    fn adapter(&self) -> &Adapter {
        self.computing_engine.adapter()
    }

    fn logical_device(&self) -> &LogicalDevice {
        self.computing_engine.logical_device()
    }
}

impl TRenderingEngine for WGPURenderingEngine {
    fn configure_surface(&mut self) {
        self.surface()
            .configure(self.device(), self.surface_configuration());
    }

    fn surface(&self) -> &wgpu::Surface {
        self.surface.surface()
    }

    fn set_surface_configuration(&mut self, surface_configuration: SurfaceConfiguration) {
        self.surface
            .set_surface_configuration(surface_configuration);
    }

    fn surface_configuration(&self) -> &SurfaceConfiguration {
        self.surface.surface_configuration()
    }

    fn surface_texture_format(&self) -> TextureFormat {
        self.surface.surface_texture_format()
    }

    fn depth_texture(&self) -> Option<&DepthTexture> {
        Some(&self.depth_texture)
    }

    fn render_pipeline(&self) -> &RenderPipeline {
        &self.render_pipeline
    }
}

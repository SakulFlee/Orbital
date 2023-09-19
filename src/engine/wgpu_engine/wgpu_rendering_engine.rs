use wgpu::{
    include_wgsl, Adapter, BlendState, ColorTargetState, ColorWrites, CompareFunction,
    CompositeAlphaMode, DepthBiasState, DepthStencilState, Device, Face, FragmentState, FrontFace,
    Instance, MultisampleState, PipelineLayoutDescriptor, PolygonMode, PresentMode, PrimitiveState,
    PrimitiveTopology, Queue, RenderPipeline, RenderPipelineDescriptor, ShaderModule, StencilState,
    Surface, SurfaceConfiguration, TextureFormat, VertexState,
};
use winit::window::Window;

use crate::engine::{
    EngineError, EngineResult, SurfaceConfigurationHelper, SurfaceHelper, TComputingEngine,
    TRenderingEngine,
};

use super::wgpu_computing_engine::WGPUComputingEngine;

pub struct WGPURenderingEngine {
    computing_engine: WGPUComputingEngine,
    surface: Surface,
    surface_texture_format: TextureFormat,
    surface_configuration: SurfaceConfiguration,
    render_pipeline: RenderPipeline,
}

impl WGPURenderingEngine {
    pub const DEPTH_FORMAT: TextureFormat = TextureFormat::Depth32Float;

    pub fn new(window: &Window) -> EngineResult<Self> {
        let instance = WGPUComputingEngine::make_instance();
        let surface = Self::make_surface(&instance, window)?;

        let computing_engine = WGPUComputingEngine::from_instance(instance, |x| {
            if x.is_surface_supported(&surface) {
                5000
            } else {
                i32::MIN
            }
        })?;

        let surface_texture_format =
            surface.find_srgb_surface_texture_format(computing_engine.get_adapter())?;

        let surface_configuration = SurfaceConfiguration::from_window(
            surface_texture_format,
            window,
            PresentMode::AutoVsync,
            CompositeAlphaMode::Auto,
        );

        let render_pipeline =
            Self::make_render_pipeline(computing_engine.get_device(), surface_texture_format)?;

        Ok(Self {
            computing_engine,
            surface,
            surface_texture_format,
            surface_configuration,
            render_pipeline,
        })
    }

    fn make_surface(instance: &Instance, window: &Window) -> EngineResult<Surface> {
        let surface = unsafe { instance.create_surface(window) }
            .map_err(|_| EngineError::CreateSurfaceError)?;
        log::debug!("Surface: {:#?}", surface);

        Ok(surface)
    }

    fn make_shader(device: &Device) -> ShaderModule {
        device.create_shader_module(include_wgsl!("../shaders/main.wgsl"))
    }

    fn make_render_pipeline(
        device: &Device,
        surface_texture_format: TextureFormat,
    ) -> EngineResult<RenderPipeline> {
        let main_shader = Self::make_shader(device);

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        Ok(device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            // Vertex shader
            vertex: VertexState {
                module: &main_shader,
                entry_point: "vs_main",
                // Vertex buffers
                buffers: &[],
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
                format: Self::DEPTH_FORMAT,
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
    fn get_instance(&self) -> &Instance {
        self.computing_engine.get_instance()
    }

    fn get_adapter(&self) -> &Adapter {
        self.computing_engine.get_adapter()
    }

    fn get_device(&self) -> &Device {
        self.computing_engine.get_device()
    }

    fn get_queue(&self) -> &Queue {
        self.computing_engine.get_queue()
    }
}

impl TRenderingEngine for WGPURenderingEngine {
    fn configure_surface(&mut self) {
        self.surface
            .configure(self.get_device(), self.get_surface_configuration());
    }

    fn get_surface(&self) -> &Surface {
        &self.surface
    }

    fn set_surface_configuration(&mut self, surface_configuration: SurfaceConfiguration) {
        self.surface_configuration = surface_configuration;
    }

    fn get_surface_configuration(&self) -> &SurfaceConfiguration {
        &self.surface_configuration
    }

    fn get_surface_texture_format(&self) -> &TextureFormat {
        &self.surface_texture_format
    }

    fn get_render_pipeline(&self) -> &RenderPipeline {
        &self.render_pipeline
    }
}

use wgpu::{
    Adapter, CompositeAlphaMode, Device, Instance, PresentMode, Queue, Surface,
    SurfaceConfiguration, TextureFormat,
};
use winit::window::Window;

use crate::{
    ComputingEngine, EngineError, EngineResult, RenderingEngine, SurfaceConfigurationHelper,
    SurfaceHelper, WGPUComputingEngine,
};

pub struct WGPURenderingEngine {
    computing_engine: WGPUComputingEngine,
    surface: Surface,
    surface_texture_format: TextureFormat,
    surface_configuration: SurfaceConfiguration,
}

impl WGPURenderingEngine {
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

        Ok(Self {
            computing_engine,
            surface,
            surface_texture_format,
            surface_configuration,
        })
    }

    fn make_surface(instance: &Instance, window: &Window) -> EngineResult<Surface> {
        let surface = unsafe { instance.create_surface(window) }
            .map_err(|_| EngineError::CreateSurfaceError)?;
        log::debug!("Surface: {:#?}", surface);

        Ok(surface)
    }
}

impl ComputingEngine for WGPURenderingEngine {
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

impl RenderingEngine for WGPURenderingEngine {
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
}

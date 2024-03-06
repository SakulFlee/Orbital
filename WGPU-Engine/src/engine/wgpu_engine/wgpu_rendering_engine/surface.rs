use wgpu::{CompositeAlphaMode, Instance, PresentMode, SurfaceConfiguration, TextureFormat};
use winit::window::Window;

use crate::engine::{
    EngineError, EngineResult, SurfaceConfigurationHelper, SurfaceHelper, TComputingEngine,
    WGPUComputingEngine,
};

pub struct Surface {
    surface: wgpu::Surface,
    surface_texture_format: TextureFormat,
    surface_configuration: SurfaceConfiguration,
}

impl Surface {
    pub fn from_window(window: &Window) -> EngineResult<(WGPUComputingEngine, Self)> {
        let instance = WGPUComputingEngine::make_instance();

        Self::from_instance_window(instance, window)
    }

    pub fn from_instance_window(
        instance: Instance,
        window: &Window,
    ) -> EngineResult<(WGPUComputingEngine, Self)> {
        let surface = Self::make_surface(&instance, window)?;

        let computing_engine = WGPUComputingEngine::from_instance(instance, |x| {
            if x.is_surface_supported(&surface) {
                5000
            } else {
                i32::MIN
            }
        })?;

        let surface_texture_format =
            surface.find_srgb_surface_texture_format(computing_engine.adapter())?;

        let surface_configuration = SurfaceConfiguration::from_window(
            surface_texture_format,
            window,
            PresentMode::AutoVsync,
            CompositeAlphaMode::Auto,
        );

        surface.configure(computing_engine.device(), &surface_configuration);

        Ok((
            computing_engine,
            Self {
                surface,
                surface_texture_format,
                surface_configuration,
            },
        ))
    }

    fn make_surface(instance: &Instance, window: &Window) -> EngineResult<wgpu::Surface> {
        let surface = unsafe { instance.create_surface(window) }
            .map_err(|_| EngineError::CreateSurfaceError)?;
        log::debug!("Surface: {:#?}", surface);

        Ok(surface)
    }

    pub fn surface(&self) -> &wgpu::Surface {
        &self.surface
    }

    pub fn surface_texture_format(&self) -> TextureFormat {
        self.surface_texture_format
    }

    pub fn surface_configuration(&self) -> &SurfaceConfiguration {
        &self.surface_configuration
    }

    pub fn set_surface_configuration(&mut self, surface_configuration: SurfaceConfiguration) {
        self.surface_configuration = surface_configuration;
    }
}

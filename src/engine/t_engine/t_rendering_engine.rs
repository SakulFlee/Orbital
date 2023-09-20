use wgpu::{
    CompositeAlphaMode, PresentMode, RenderPipeline, Surface, SurfaceConfiguration, SurfaceTexture,
    TextureFormat, TextureView,
};

use crate::engine::{EngineError, EngineResult, TextureHelper};

use super::TComputingEngine;

pub trait TRenderingEngine: TComputingEngine {
    fn configure_surface(&mut self);
    fn reconfigure_surface(&mut self) {
        self.configure_surface()
    }

    fn change_window_size(&mut self, size: (u32, u32)) {
        let mut new_surface_configuration = self.get_surface_configuration().clone();

        new_surface_configuration.width = size.0;
        new_surface_configuration.height = size.1;

        self.set_surface_configuration(new_surface_configuration);
        self.reconfigure_surface();
    }

    fn change_vsync(&mut self, present_mode: PresentMode) {
        let mut new_surface_configuration = self.get_surface_configuration().clone();

        new_surface_configuration.present_mode = present_mode;

        self.set_surface_configuration(new_surface_configuration);
        self.reconfigure_surface();
    }

    fn change_composite_alpha(&mut self, alpha_mode: CompositeAlphaMode) {
        let mut new_surface_configuration = self.get_surface_configuration().clone();

        new_surface_configuration.alpha_mode = alpha_mode;

        self.set_surface_configuration(new_surface_configuration);
        self.reconfigure_surface();
    }

    fn get_surface(&self) -> &Surface;
    fn get_surface_configuration(&self) -> &SurfaceConfiguration;
    fn set_surface_configuration(&mut self, surface_configuration: SurfaceConfiguration);
    fn get_surface_texture_format(&self) -> TextureFormat;

    fn get_surface_texture(&self) -> EngineResult<SurfaceTexture> {
        Ok(self
            .get_surface()
            .get_current_texture()
            .map_err(|e| EngineError::SurfaceError(e))?)
    }

    fn get_surface_texture_view(&self) -> EngineResult<TextureView> {
        Ok(self.get_surface_texture()?.make_texture_view())
    }

    fn get_render_pipeline(&self) -> &RenderPipeline;
}

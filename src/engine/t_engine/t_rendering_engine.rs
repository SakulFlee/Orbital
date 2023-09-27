use wgpu::{
    CompositeAlphaMode, PresentMode, RenderPipeline, Surface, SurfaceConfiguration, SurfaceTexture,
    TextureFormat, TextureView,
};

use crate::engine::{DepthTexture, EngineError, EngineResult, TextureHelper};

use super::TComputingEngine;

pub trait TRenderingEngine: TComputingEngine {
    fn configure_surface(&mut self);
    fn reconfigure_surface(&mut self) {
        self.configure_surface()
    }

    fn change_window_size(&mut self, size: (u32, u32)) {
        let mut new_surface_configuration = self.surface_configuration().clone();

        new_surface_configuration.width = size.0;
        new_surface_configuration.height = size.1;

        self.set_surface_configuration(new_surface_configuration);
        self.reconfigure_surface();
    }

    fn change_vsync(&mut self, present_mode: PresentMode) {
        let mut new_surface_configuration = self.surface_configuration().clone();

        new_surface_configuration.present_mode = present_mode;

        self.set_surface_configuration(new_surface_configuration);
        self.reconfigure_surface();
    }

    fn change_composite_alpha(&mut self, alpha_mode: CompositeAlphaMode) {
        let mut new_surface_configuration = self.surface_configuration().clone();

        new_surface_configuration.alpha_mode = alpha_mode;

        self.set_surface_configuration(new_surface_configuration);
        self.reconfigure_surface();
    }

    fn surface(&self) -> &Surface;
    fn surface_configuration(&self) -> &SurfaceConfiguration;
    fn set_surface_configuration(&mut self, surface_configuration: SurfaceConfiguration);
    fn surface_texture_format(&self) -> TextureFormat;

    fn surface_texture(&self) -> EngineResult<SurfaceTexture> {
        self.surface()
            .get_current_texture()
            .map_err(EngineError::SurfaceError)
    }

    fn surface_texture_view(&self) -> EngineResult<TextureView> {
        Ok(self.surface_texture()?.make_texture_view())
    }

    fn depth_texture(&self) -> Option<&DepthTexture>;

    fn render_pipeline(&self) -> &RenderPipeline;
}

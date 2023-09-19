use wgpu::{Adapter, Surface, TextureFormat};

use crate::{EngineError, EngineResult};

pub trait SurfaceHelper {
    fn get_surface_texture_formats(&self, adapter: &Adapter) -> Vec<TextureFormat>;
    fn find_surface_texture_format<P>(
        &self,
        adapter: &Adapter,
        predicate: P,
    ) -> EngineResult<TextureFormat>
    where
        P: Fn(&&TextureFormat) -> bool;
    fn find_srgb_surface_texture_format(&self, adapter: &Adapter) -> EngineResult<TextureFormat>;
}

impl SurfaceHelper for Surface {
    fn get_surface_texture_formats(&self, adapter: &Adapter) -> Vec<TextureFormat> {
        self.get_capabilities(adapter).formats
    }

    fn find_surface_texture_format<P>(
        &self,
        adapter: &Adapter,
        predicate: P,
    ) -> EngineResult<TextureFormat>
    where
        P: Fn(&&TextureFormat) -> bool,
    {
        Ok(self
            .get_surface_texture_formats(adapter)
            .iter()
            .find(predicate)
            .cloned()
            .ok_or(EngineError::NoMatch)?)
    }

    fn find_srgb_surface_texture_format(&self, adapter: &Adapter) -> EngineResult<TextureFormat> {
        self.find_surface_texture_format(adapter, |x| x.is_srgb())
    }
}

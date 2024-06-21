use std::{sync::OnceLock, time::Instant};

use cgmath::Vector2;
use log::{debug, info, warn};
use wgpu::{Device, Queue, SurfaceConfiguration, TextureView};
use winit::event_loop::EventLoop;

use crate::{
    app::{App, AppRuntime},
    error::Error,
    renderer::Renderer,
    resources::realizations::{Material, Pipeline, Texture},
    timer::Timer,
};

use super::{CacheSettings, Game, GameSettings, World};

pub struct GameRuntime<GameImpl: Game, RendererImpl: Renderer> {
    game: GameImpl,
    world: World,
    timer: Timer,
    renderer: RendererImpl,
    pipeline_cleanup_timer: Instant,
    material_cleanup_timer: Instant,
    texture_cleanup_timer: Instant,
}

pub static mut PIPELINE_CACHE_SETTINGS: OnceLock<CacheSettings> = OnceLock::new();
pub static mut MATERIAL_CACHE_SETTINGS: OnceLock<CacheSettings> = OnceLock::new();
pub static mut TEXTURE_CACHE_SETTINGS: OnceLock<CacheSettings> = OnceLock::new();

impl<GameImpl: Game, RendererImpl: Renderer> GameRuntime<GameImpl, RendererImpl> {
    pub fn liftoff(event_loop: EventLoop<()>, settings: GameSettings) -> Result<(), Error> {
        info!("Akimo-Project: Game Runtime");
        info!(" --- @SakulFlee --- ");

        #[cfg(feature = "dev_build")]
        warn!("⚠️ THIS IS A DEV BUILD ⚠️");

        unsafe {
            PIPELINE_CACHE_SETTINGS.get_or_init(|| settings.pipeline_cache);
            MATERIAL_CACHE_SETTINGS.get_or_init(|| settings.material_cache);
            TEXTURE_CACHE_SETTINGS.get_or_init(|| settings.texture_cache);
        }

        AppRuntime::<GameRuntime<GameImpl, RendererImpl>>::__liftoff(
            event_loop,
            settings.app_settings,
        )
    }

    fn do_cleanup(&mut self, device: &Device, queue: &Queue) {
        self.do_pipeline_cache_cleanup(device, queue);
        self.do_material_cache_cleanup();
        self.do_texture_cache_cleanup();
    }

    fn do_pipeline_cache_cleanup(&mut self, device: &Device, queue: &Queue) {
        let pipeline_cache_settings = unsafe { PIPELINE_CACHE_SETTINGS.get().unwrap() };

        if self.pipeline_cleanup_timer.elapsed() >= pipeline_cache_settings.cleanup_interval {
            info!("Pipeline cache cleanup started!");

            // Reuse variable as performance measure
            self.pipeline_cleanup_timer = Instant::now();

            // Cache access
            let cache = Pipeline::prepare_cache_access(None, device, queue);

            // Run cleanup
            let change = cache.cleanup(pipeline_cache_settings.retain_period);
            info!("Pipeline {}", change);

            // Print out duration
            debug!(
                "Pipeline Cache Cleanup took {}ms!",
                self.pipeline_cleanup_timer.elapsed().as_millis()
            );

            // Reset timer
            self.pipeline_cleanup_timer = Instant::now();
        }
    }

    fn do_material_cache_cleanup(&mut self) {
        let material_cache_settings = unsafe { MATERIAL_CACHE_SETTINGS.get().unwrap() };

        if self.material_cleanup_timer.elapsed() >= material_cache_settings.cleanup_interval {
            info!("Material cache cleanup started!");

            // Reuse variable as performance measure
            self.material_cleanup_timer = Instant::now();

            // Cache access
            let cache = Material::prepare_cache_access();

            // Run cleanup
            let change = cache.cleanup(material_cache_settings.retain_period);
            info!("Material {}", change);

            // Print out duration
            debug!(
                "Material Cache Cleanup took {}ms!",
                self.material_cleanup_timer.elapsed().as_millis()
            );

            // Reset timer
            self.material_cleanup_timer = Instant::now();
        }
    }

    fn do_texture_cache_cleanup(&mut self) {
        let texture_cache_settings = unsafe { TEXTURE_CACHE_SETTINGS.get().unwrap() };

        if self.texture_cleanup_timer.elapsed() >= texture_cache_settings.cleanup_interval {
            info!("Texture cache cleanup started!");

            // Reuse variable as performance measure
            self.texture_cleanup_timer = Instant::now();

            // Cache access
            let cache = Texture::prepare_cache_access();

            // Run cleanup
            let change = cache.cleanup(texture_cache_settings.retain_period);
            info!("Texture {}", change);

            // Print out duration
            debug!(
                "Texture Cache Cleanup took {}ms!",
                self.texture_cleanup_timer.elapsed().as_millis()
            );

            // Reset timer
            self.texture_cleanup_timer = Instant::now();
        }
    }
}

impl<GameImpl: Game, RendererImpl: Renderer> App for GameRuntime<GameImpl, RendererImpl> {
    fn init(config: &SurfaceConfiguration, device: &Device, queue: &Queue) -> Self
    where
        Self: Sized,
    {
        Self {
            game: GameImpl::init(),
            world: World::default(),
            timer: Timer::new(),
            renderer: RendererImpl::new(
                config.format,
                (config.width, config.height).into(),
                device,
                queue,
            ),
            pipeline_cleanup_timer: Instant::now(),
            material_cleanup_timer: Instant::now(),
            texture_cleanup_timer: Instant::now(),
        }
    }

    fn on_resize(&mut self, new_resolution: Vector2<u32>, device: &Device, queue: &Queue)
    where
        Self: Sized,
    {
        self.renderer
            .change_resolution(new_resolution, device, queue);
    }

    fn on_update(&mut self)
    where
        Self: Sized,
    {
        let delta_time = self.timer.cycle_delta_time();

        self.game
            .cycle(delta_time, &mut self.world);

        self.renderer.update(delta_time);
    }

    fn on_render(&mut self, target_view: &TextureView, device: &Device, queue: &Queue)
    where
        Self: Sized,
    {
        self.world.update(device, queue);

        self.renderer
            .render(target_view, device, queue, self.world.composition.models());

        if let Some((delta_time, fps)) = self.timer.tick() {
            debug!("FPS: {fps}");
            debug!("Tick  Delta: {} ms", delta_time);

            self.do_cleanup(device, queue);
        }
    }
}

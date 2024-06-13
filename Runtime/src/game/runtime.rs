use std::{sync::OnceLock, time::Instant};

use log::{debug, info, warn};
use wgpu::{Device, Queue, SurfaceConfiguration, TextureView};
use winit::event_loop::EventLoop;

use crate::{
    app::{App, AppRuntime},
    error::Error,
    resources::realizations::Pipeline,
    server::RenderServer,
    timer::Timer,
};

use super::{CacheSettings, Game, GameSettings};

pub struct GameRuntime<GameImpl: Game> {
    game: GameImpl,
    timer: Timer,
    render_server: RenderServer,
    pipeline_cleanup_timer: Instant,
}

pub static mut PIPELINE_CACHE_SETTINGS: OnceLock<CacheSettings> = OnceLock::new();

impl<GameImpl: Game> GameRuntime<GameImpl> {
    pub fn liftoff(event_loop: EventLoop<()>, settings: GameSettings) -> Result<(), Error> {
        info!("Akimo-Project: Game Runtime");
        info!(" --- @SakulFlee --- ");

        #[cfg(feature = "dev_build")]
        warn!("⚠️ THIS IS A DEV BUILD ⚠️");

        unsafe {
            PIPELINE_CACHE_SETTINGS.get_or_init(|| settings.pipeline_cache);
        }

        AppRuntime::<GameRuntime<GameImpl>>::__liftoff(event_loop, settings.app_settings)
    }

    fn check_pipeline_cleanup_cycle(&mut self, device: &Device, queue: &Queue) {
        let pipeline_cache_settings = unsafe { PIPELINE_CACHE_SETTINGS.get().unwrap() };

        if self.pipeline_cleanup_timer.elapsed() >= pipeline_cache_settings.cleanup_interval {
            info!("Pipeline cache cleanup started!");

            let cache = Pipeline::prepare_cache_access(None, device, queue);

            cache.cleanup(pipeline_cache_settings.retain_period);
        }
    }
}

impl<GameImpl: Game> App for GameRuntime<GameImpl> {
    fn init(config: &SurfaceConfiguration, device: &Device, queue: &Queue) -> Self
    where
        Self: Sized,
    {
        Self {
            game: GameImpl::init(),
            timer: Timer::new(),
            render_server: RenderServer::new(
                config.format,
                (config.width, config.height).into(),
                device,
                queue,
            ),
            pipeline_cleanup_timer: Instant::now(),
        }
    }

    fn on_resize(&mut self, new_size: cgmath::Vector2<u32>, device: &Device, queue: &Queue)
    where
        Self: Sized,
    {
        self.render_server
            .change_depth_texture_resolution(new_size, device, queue)
    }

    fn on_update(&mut self)
    where
        Self: Sized,
    {
        self.game
            .cycle(self.timer.cycle_delta_time(), &mut self.render_server);
    }

    fn on_render(&mut self, view: &TextureView, device: &Device, queue: &Queue)
    where
        Self: Sized,
    {
        if let Some((delta_time, fps)) = self.timer.tick() {
            debug!("FPS: {fps}");
            debug!("Tick  Delta: {} ms", delta_time);

            self.check_pipeline_cleanup_cycle(device, queue);
        }

        self.render_server.render(view, device, queue);
    }
}

use log::{debug, info, warn};
use wgpu::{Device, Queue, SurfaceConfiguration, TextureView};
use winit::event_loop::EventLoop;

use crate::{
    app::{App, AppRuntime, RuntimeSettings},
    error::Error,
    server::RenderServer,
    timer::Timer,
};

use super::Game;

pub struct GameRuntime<GameImpl: Game> {
    game: GameImpl,
    timer: Timer,
    render_server: RenderServer,
}

impl<GameImpl: Game> GameRuntime<GameImpl> {
    pub fn liftoff(event_loop: EventLoop<()>, settings: RuntimeSettings) -> Result<(), Error> {
        info!("Akimo-Project: Game Runtime");
        info!(" --- @SakulFlee --- ");

        #[cfg(feature = "dev_build")]
        warn!("⚠️ THIS IS A DEV BUILD ⚠️");

        AppRuntime::<GameRuntime<GameImpl>>::__liftoff(event_loop, settings)
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
        }

        self.render_server.render(view, device, queue);
    }
}

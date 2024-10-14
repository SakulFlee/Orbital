use std::{sync::OnceLock, time::Instant};

use cgmath::Vector2;
use log::{debug, info};
use wgpu::{Device, Queue, SurfaceConfiguration, TextureView};
use winit::{
    event::{DeviceEvent, DeviceId},
    event_loop::EventLoop,
};

use crate::{
    app::{App, AppChange, AppRuntime, InputEvent},
    error::Error,
    renderer::Renderer,
    resources::realizations::{Material, Pipeline},
    timer::Timer,
};

use super::{CacheSettings, Game, GameSettings, World};

pub struct GameRuntime<GameImpl: Game, RendererImpl: Renderer> {
    game: GameImpl,
    game_startup_complete: bool,
    world: World,
    timer: Timer,
    renderer: RendererImpl,
    pipeline_cleanup_timer: Instant,
    material_cleanup_timer: Instant,
}

pub static mut PIPELINE_CACHE_SETTINGS: OnceLock<CacheSettings> = OnceLock::new();
pub static mut MATERIAL_CACHE_SETTINGS: OnceLock<CacheSettings> = OnceLock::new();

impl<GameImpl: Game, RendererImpl: Renderer> GameRuntime<GameImpl, RendererImpl> {
    pub fn liftoff(event_loop: EventLoop<()>, settings: GameSettings) -> Result<(), Error> {
        info!("Orbital: Game Runtime");
        info!(" --- @SakulFlee --- ");

        unsafe {
            PIPELINE_CACHE_SETTINGS.get_or_init(|| settings.pipeline_cache);
            MATERIAL_CACHE_SETTINGS.get_or_init(|| settings.material_cache);
        }

        AppRuntime::<GameRuntime<GameImpl, RendererImpl>>::__liftoff(
            event_loop,
            settings.app_settings,
        )
    }

    fn do_cleanup(&mut self, device: &Device, queue: &Queue) {
        self.do_pipeline_cache_cleanup(device, queue);
        self.do_material_cache_cleanup();
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
}

impl<GameImpl: Game, RendererImpl: Renderer> App for GameRuntime<GameImpl, RendererImpl> {
    fn init(config: &SurfaceConfiguration, device: &Device, queue: &Queue) -> Self
    where
        Self: Sized,
    {
        Self {
            game: GameImpl::init(),
            game_startup_complete: false,
            world: World::new(),
            timer: Timer::new(),
            renderer: RendererImpl::new(
                config.format,
                (config.width, config.height).into(),
                device,
                queue,
            ),
            pipeline_cleanup_timer: Instant::now(),
            material_cleanup_timer: Instant::now(),
        }
    }

    fn on_resize(&mut self, new_resolution: Vector2<u32>, device: &Device, queue: &Queue)
    where
        Self: Sized,
    {
        self.renderer
            .change_resolution(new_resolution, device, queue);
    }

    fn on_focus_change(&mut self, focused: bool)
    where
        Self: Sized,
    {
        self.world.on_focus_change(focused);
    }

    fn on_input(&mut self, input_event: &InputEvent)
    where
        Self: Sized,
    {
        self.world.on_input_event(input_event)
    }

    fn on_update(&mut self) -> Option<Vec<AppChange>>
    where
        Self: Sized,
    {
        let delta_time = self.timer.cycle_delta_time();

        if !self.game_startup_complete {
            self.game.on_startup(&mut self.world);
            self.game_startup_complete = true;
        }

        let app_changes = self.world.update(delta_time);

        // TODO: Needed?
        self.renderer.update(delta_time);

        if app_changes.is_empty() {
            None
        } else {
            Some(app_changes)
        }
    }

    fn on_render(&mut self, target_view: &TextureView, device: &Device, queue: &Queue)
    where
        Self: Sized,
    {
        self.world.prepare_render(device, queue);

        self.renderer
            .render(target_view, device, queue, &self.world);

        if let Some((delta_time, fps)) = self.timer.tick() {
            debug!("FPS: {fps}");
            debug!("Tick  Delta: {} ms", delta_time);

            self.do_cleanup(device, queue);
        }
    }

    fn on_device_event(&mut self, device_id: DeviceId, event: DeviceEvent)
    where
        Self: Sized,
    {
        if let DeviceEvent::MouseMotion { delta } = event {
            let input_event = InputEvent::MouseMovedDelta { device_id, delta };
            self.world.on_input_event(&input_event);
        }
    }
}

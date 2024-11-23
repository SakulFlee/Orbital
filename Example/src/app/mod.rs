use std::time::Instant;

use orbital::{
    app::{App, AppChange, InputEvent},
    cgmath::Vector2,
    game::{CacheSettings, World, WorldChange},
    log::{debug, info, warn},
    renderer::Renderer,
    resources::realizations::{Material, Pipeline},
    timer::Timer,
    wgpu::{Device, Queue, SurfaceConfiguration, TextureView},
};

use crate::game::elements::{
    camera::Camera, damaged_helmet::DamagedHelmet, debug_world_environment::DebugWorldEnvironment,
    lights::Lights, pbr_spheres::PBRSpheres, ping_pong::PingPongElement,
};

pub struct MyApp<RendererImpl: Renderer + Send> {
    renderer: Option<RendererImpl>,
    world: World,
    timer: Timer,
    cache_settings_pipelines: CacheSettings,
    cache_settings_materials: CacheSettings,
    cache_timer_pipelines: Instant,
    cache_timer_materials: Instant,
}

impl<RenderImpl: Renderer + Send> MyApp<RenderImpl> {
    pub fn new(
        cache_settings_pipelines: CacheSettings,
        cache_settings_materials: CacheSettings,
    ) -> Self {
        Self {
            renderer: None,
            world: World::new(),
            timer: Timer::new(),
            cache_settings_pipelines,
            cache_settings_materials,
            cache_timer_pipelines: Instant::now(),
            cache_timer_materials: Instant::now(),
        }
    }

    fn on_startup(&mut self) {
        // Debug
        self.world
            .process_world_change(WorldChange::SpawnElement(Box::new(
                DebugWorldEnvironment::new(),
            )));

        // Camera & Lights
        self.world
            .process_world_change(WorldChange::SpawnElement(Box::new(Camera::new())));
        self.world
            .process_world_change(WorldChange::SpawnElement(Box::new(Lights {})));

        // Ping Pong
        self.world
            .process_world_change(WorldChange::SpawnElement(Box::new(PingPongElement::new(
                true,
            ))));
        self.world
            .process_world_change(WorldChange::SpawnElement(Box::new(PingPongElement::new(
                false,
            ))));

        // Models
        self.world
            .process_world_change(WorldChange::SpawnElement(Box::new(PBRSpheres {})));
        self.world
            .process_world_change(WorldChange::SpawnElement(Box::new(DamagedHelmet {})));
    }

    fn cache_cleanup(&mut self) {
        self.cache_cleanup_pipelines();
        self.cache_cleanup_materials();
    }

    fn cache_cleanup_pipelines(&mut self) {
        if self.cache_timer_pipelines.elapsed() < self.cache_settings_pipelines.cleanup_interval {
            return;
        }

        // Cache access
        let cache = Pipeline::prepare_cache_access(None);

        // Run cleanup
        let change = cache.cleanup(self.cache_settings_pipelines.retain_period);
        info!("Pipeline {}", change);

        // Print out duration
        debug!(
            "Pipeline Cache Cleanup took {}ms!",
            self.cache_timer_pipelines.elapsed().as_millis()
        );

        self.cache_timer_pipelines = Instant::now();
    }

    fn cache_cleanup_materials(&mut self) {
        if self.cache_timer_materials.elapsed() < self.cache_settings_materials.cleanup_interval {
            return;
        }

        // Cache access
        let cache = Material::prepare_cache_access();

        // Run cleanup
        let change = cache.cleanup(self.cache_settings_materials.retain_period);
        info!("Material {}", change);

        // Print out duration
        debug!(
            "Material Cache Cleanup took {}ms!",
            self.cache_timer_materials.elapsed().as_millis()
        );

        self.cache_timer_materials = Instant::now();
    }
}

impl<RenderImpl: Renderer + Send> App for MyApp<RenderImpl> {
    async fn on_resume(&mut self, config: &SurfaceConfiguration, device: &Device, queue: &Queue) {
        self.renderer = Some(RenderImpl::new(
            config.format,
            Vector2::new(config.width, config.height),
            device,
            queue,
        ));

        if self.world.models().is_empty() {
            self.on_startup();
        }
    }

    async fn on_suspend(&mut self) {
        self.renderer = None;
    }

    async fn on_resize(&mut self, new_size: Vector2<u32>, device: &Device, queue: &Queue)
    where
        Self: Sized,
    {
        if let Some(renderer) = &mut self.renderer {
            renderer.change_resolution(new_size, device, queue);
        } else {
            warn!("Received resize event, but Renderer doesn't exist (yet?)");
        }
    }

    async fn on_focus_change(&mut self, focused: bool)
    where
        Self: Sized,
    {
        self.world.on_focus_change(focused);
    }

    async fn on_input(&mut self, input_event: &InputEvent) -> ()
    where
        Self: Sized,
    {
        self.world.on_input_event(input_event);
    }

    async fn on_update(&mut self) -> Option<Vec<AppChange>>
    where
        Self: Sized,
    {
        let delta_time = self.timer.cycle_delta_time();
        let app_changes = self.world.update(delta_time);

        // TODO: Needed?
        if let Some(renderer) = &mut self.renderer {
            renderer.update(delta_time);
        }

        (!app_changes.is_empty()).then_some(app_changes)
    }

    async fn on_render(&mut self, target_view: &TextureView, device: &Device, queue: &Queue) -> ()
    where
        Self: Sized,
    {
        self.world.prepare_render(device, queue);

        if let Some(renderer) = &mut self.renderer {
            renderer.render(target_view, device, queue, &self.world);
        }

        if let Some((delta_time, fps)) = self.timer.tick() {
            debug!("FPS: {fps}");
            debug!("Tick  Delta: {} ms", delta_time);

            self.cache_cleanup();
        }
    }
}

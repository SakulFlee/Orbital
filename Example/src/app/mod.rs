use std::time::Instant;

use orbital::{
    app::{App, AppChange},
    cgmath::Vector2,
    input::InputState,
    log::{debug, info, warn},
    renderer::Renderer,
    resources::realizations::{Material, Pipeline},
    wgpu::{Device, Queue, SurfaceConfiguration, TextureView},
    world::{World, WorldChange},
};

mod cache_settings;
pub use cache_settings::*;

mod elements;
use elements::*;

pub struct MyApp<RendererImpl: Renderer + Send> {
    renderer: Option<RendererImpl>,
    world: World,
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
            cache_settings_pipelines,
            cache_settings_materials,
            cache_timer_pipelines: Instant::now(),
            cache_timer_materials: Instant::now(),
        }
    }

    async fn on_startup(&mut self) {
        // Debug
        self.world
            .process_world_change(WorldChange::SpawnElement(Box::new(
                DebugWorldEnvironment::new(),
            )))
            .await;

        // Camera & Lights
        self.world
            .process_world_change(WorldChange::SpawnElement(Box::new(Camera::new())))
            .await;
        self.world
            .process_world_change(WorldChange::SpawnElement(Box::new(Lights {})))
            .await;

        // Ping Pong
        self.world
            .process_world_change(WorldChange::SpawnElement(Box::new(PingPongElement::new(
                true,
            ))))
            .await;
        self.world
            .process_world_change(WorldChange::SpawnElement(Box::new(PingPongElement::new(
                false,
            ))))
            .await;

        // Models
        self.world
            .process_world_change(WorldChange::SpawnElement(Box::new(PBRSpheres {})))
            .await;
        self.world
            .process_world_change(WorldChange::SpawnElement(Box::new(DamagedHelmet {})))
            .await;
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

        if self.world.model_store().is_empty() {
            self.on_startup().await;
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

    async fn on_update(
        &mut self,
        input_state: &InputState,
        delta_time: f64,
        cycle: Option<(f64, u64)>,
    ) -> Option<Vec<AppChange>>
    where
        Self: Sized,
    {
        let app_changes = self.world.update(delta_time, input_state).await;

        // TODO: Needed?
        if let Some(renderer) = &mut self.renderer {
            renderer.update(delta_time);
        }

        if cycle.is_some() {
            self.cache_cleanup();
        }

        (!app_changes.is_empty()).then_some(app_changes)
    }

    async fn on_render(&mut self, target_view: &TextureView, device: &Device, queue: &Queue)
    where
        Self: Sized,
    {
        self.world.prepare_render(device, queue);

        if let Some(renderer) = &mut self.renderer {
            renderer.render(target_view, device, queue, &self.world);
        }
    }
}

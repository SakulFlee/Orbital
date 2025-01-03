use orbital::{
    app::{App, AppChange},
    cgmath::Vector2,
    input::InputState,
    log::warn,
    renderer::Renderer,
    wgpu::{Device, Queue, SurfaceConfiguration, TextureView},
    world::{World, WorldChange},
};

mod cache_settings;
pub use cache_settings::*;

mod elements;
use elements::*;

use crate::entrypoint::NAME;

pub struct MyApp<RendererImpl: Renderer + Send> {
    renderer: Option<RendererImpl>,
    world: World,
}

impl<RenderImpl: Renderer + Send> Default for MyApp<RenderImpl> {
    fn default() -> Self {
        Self::new()
    }
}

impl<RenderImpl: Renderer + Send> MyApp<RenderImpl> {
    pub fn new() -> Self {
        Self {
            renderer: None,
            world: World::new(),
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
}

impl<RenderImpl: Renderer + Send> App for MyApp<RenderImpl> {
    async fn on_resume(&mut self, config: &SurfaceConfiguration, device: &Device, queue: &Queue) {
        self.renderer = Some(RenderImpl::new(
            config.format,
            Vector2::new(config.width, config.height),
            device,
            queue,
            NAME,
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
            renderer.change_resolution(new_size, device, queue).await;
        } else {
            warn!("Received resize event, but Renderer doesn't exist (yet?)");
        }
    }

    async fn on_update(
        &mut self,
        input_state: &InputState,
        delta_time: f64,
        _cycle: Option<(f64, u64)>,
    ) -> Option<Vec<AppChange>>
    where
        Self: Sized,
    {
        let app_changes = self.world.update(delta_time, input_state).await;

        (!app_changes.is_empty()).then_some(app_changes)
    }

    async fn on_render(&mut self, target_view: &TextureView, device: &Device, queue: &Queue)
    where
        Self: Sized,
    {
        self.world.prepare_render(device, queue);

        if let Some(renderer) = &mut self.renderer {
            renderer
                .render(target_view, device, queue, &self.world)
                .await;
        }
    }
}

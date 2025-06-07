use orbital::{
    app::{input::InputState, App, RuntimeEvent},
    cgmath::Vector2,
    element::{ElementEvent, ElementStore, Event, WorldEvent},
    logging::warn,
    renderer::Renderer,
    wgpu::{Device, Queue, SurfaceConfiguration, TextureView},
    world::World,
};

mod cache_settings;
pub use cache_settings::*;

mod elements;
use elements::*;

pub struct MyApp {
    element_store: ElementStore,
    world: World,
    queue_events: Vec<Event>,
    renderer: Option<Renderer>,
}

impl Default for MyApp {
    fn default() -> Self {
        Self::new()
    }
}

impl MyApp {
    pub fn new() -> Self {
        Self {
            element_store: ElementStore::new(),
            world: World::new(),
            queue_events: Vec::new(),
            renderer: None,
        }
    }

    async fn on_startup(&mut self) {
        // // Debug
        // self.world
        //     .process_world_change(WorldChange::SpawnElement(Box::new(
        //         DebugWorldEnvironment::new(),
        //     )))
        //     .await;
        // self.world
        //     .process_world_change(WorldChange::SpawnElement(Box::new(
        //         DebugController::default(),
        //     )))
        //     .await;

        // // Camera & Lights
        // self.world
        //     .process_world_change(WorldChange::SpawnElement(Box::new(Camera::new())))
        //     .await;
        // self.world
        //     .process_world_change(WorldChange::SpawnElement(Box::new(Lights {})))
        //     .await;

        // // Ping Pong
        // self.world
        //     .process_world_change(WorldChange::SpawnElement(Box::new(PingPongElement::new(
        //         true,
        //     ))))
        //     .await;
        // self.world
        //     .process_world_change(WorldChange::SpawnElement(Box::new(PingPongElement::new(
        //         false,
        //     ))))
        //     .await;

        // // Models
        // self.world
        //     .process_world_change(WorldChange::SpawnElement(Box::new(PBRSpheres {})))
        //     .await;
        // self.world
        //     .process_world_change(WorldChange::SpawnElement(Box::new(DamagedHelmet {})))
        //     .await;
    }
}

impl App for MyApp {
    async fn on_resume(&mut self, config: &SurfaceConfiguration, device: &Device, queue: &Queue) {
        self.renderer = Some(Renderer::new(
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
        _cycle: Option<(f64, u64)>,
    ) -> Option<Vec<RuntimeEvent>>
    where
        Self: Sized,
    {
        let mut events = self.element_store.update(delta_time, input_state).await;
        let old_events = self.queue_events.drain(0..self.queue_events.len());

        events.extend(old_events);

        let mut world_events = Vec::<WorldEvent>::new();
        let mut element_events = Vec::<ElementEvent>::new();
        let mut runtime_events = Vec::<RuntimeEvent>::new();

        for event in events {
            match event {
                Event::World(world_event) => {
                    world_events.push(world_event);
                }
                Event::Element(element_event) => {
                    element_events.push(element_event);
                }
                Event::App(runtime_event) => {
                    runtime_events.push(runtime_event);
                    // TODO Should be Runtime not App Events},
                }
            }
        }

        self.world.update(world_events);
        self.element_store.process_events(element_events);

        (!runtime_events.is_empty()).then_some(runtime_events)
    }

    async fn on_render(&mut self, target_view: &TextureView, device: &Device, queue: &Queue)
    where
        Self: Sized,
    {
        if let Some(renderer) = &mut self.renderer {
            self.world.prepare_render(device);

            let world_environment = self.world.environment_store().world_environment();
            let models = Vec::new();

            renderer
                .render(target_view, world_environment, models, device, queue)
                .await;
        }
    }
}

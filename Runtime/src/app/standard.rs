use crate::element::Element;
use crate::{
    app::{input::InputState, App, AppEvent},
    cgmath::Vector2,
    element::{ElementEvent, ElementStore, Event, WorldEvent},
    logging::warn,
    renderer::Renderer,
    wgpu::{Device, Queue, SurfaceConfiguration, TextureView},
    world::World,
};
use async_std::task::block_on;
use std::time::{Duration, Instant};

#[derive(Default)]
pub struct StandardApp {
    pub(crate) element_store: ElementStore,
    pub(crate) world: World,
    pub(crate) queue_events: Vec<Event>,
    pub(crate) renderer: Option<Renderer>,
    #[cfg(feature = "standard_app_detect_no_more_elements")]
    pub(crate) empty_since: Option<Instant>,
}

impl StandardApp {
    pub fn with_initial_elements(elements: Vec<Box<dyn Element + Send + Sync>>) -> Self {
        let mut s = Self::default();

        let events = elements
            .into_iter()
            .map(ElementEvent::Spawn)
            .collect::<Vec<_>>();
        let new_events = block_on(s.element_store.process_events(events));
        s.queue_events.extend(new_events);

        s
    }
}

impl App for StandardApp {
    fn new() -> Self {
        panic!("Do not call StandardApp::new() directly, use App::new() instead!")
    }

    async fn on_startup(&mut self) {
        if self.element_store.element_count() == 0 {
            panic!("StandardApp requires at least one element to be spawned @Startup! Make sure to use StandardApp::with_initial_elements() to initialize with elements.");
        }
    }

    async fn on_resume(&mut self, config: &SurfaceConfiguration, device: &Device, queue: &Queue) {
        self.renderer = Some(Renderer::new(
            config.format,
            Vector2::new(config.width, config.height),
            device,
            queue,
        ));

        if self.world.model_store_mut().is_empty() {
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
    ) -> Option<Vec<AppEvent>>
    where
        Self: Sized,
    {
        let mut events = self.element_store.update(delta_time, input_state).await;
        let old_events = self.queue_events.drain(0..self.queue_events.len());

        events.extend(old_events);

        let mut world_events = Vec::<WorldEvent>::new();
        let mut element_events = Vec::<ElementEvent>::new();
        let mut app_events = Vec::<AppEvent>::new();

        for event in events {
            match event {
                Event::World(world_event) => {
                    world_events.push(world_event);
                }
                Event::Element(element_event) => {
                    element_events.push(element_event);
                }
                Event::App(app_event) => {
                    app_events.push(app_event);
                }
            }
        }

        // Kick off world future to process world updates while we handle other things
        let world_future = self.world.update(world_events);

        let new_events = self.element_store.process_events(element_events).await;
        self.queue_events.extend(new_events);

        #[cfg(feature = "standard_app_detect_no_more_elements")]
        {
            if self.element_store.element_count() == 0 {
                if let Some(since) = self.empty_since {
                    if since.elapsed() >= Duration::from_secs(5) {
                        warn!("No more elements present, requesting app closure ...");
                        app_events.push(AppEvent::RequestAppClosure);
                    }
                } else {
                    self.empty_since = Some(Instant::now());
                }
            } else if self.empty_since.is_some() {
                self.empty_since = None;
            }
        }

        // Await world future before we need access to the world again.
        world_future.await;

        // TODO: REMOVE
        let model_ids = self
            .world
            .model_store_mut()
            .get_bounding_boxes()
            .keys()
            .copied()
            .collect();
        self.world
            .model_store_mut()
            .flag_realization(model_ids, false);

        (!app_events.is_empty()).then_some(app_events)
    }

    async fn on_render(&mut self, target_view: &TextureView, device: &Device, queue: &Queue)
    where
        Self: Sized,
    {
        if let Some(renderer) = &mut self.renderer {
            self.world
                .prepare_render(renderer.surface_texture_format(), device, queue);

            let (world_bind_group_option, world_environment_option, models) =
                self.world.retrieve_render_resources();
            let world_bind_group = match world_bind_group_option {
                Some(x) => x,
                None => {
                    warn!("No World BindGroup found! Something went wrong. This is a bug.");
                    return;
                }
            };

            renderer
                .render(
                    target_view,
                    world_bind_group,
                    world_environment_option,
                    models,
                    device,
                    queue,
                )
                .await;
        }
    }
}

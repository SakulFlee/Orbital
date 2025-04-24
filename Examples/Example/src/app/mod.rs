use orbital::{
    app::{input::InputState, App, RuntimeEvent},
    cgmath::Vector2,
    element::{ElementEvent, ElementStore, Event},
    logging::warn,
    physics::{Physics, PhysicsEvent},
    wgpu::{Device, Queue, SurfaceConfiguration, TextureView},
};

mod cache_settings;
pub use cache_settings::*;

mod elements;
use elements::*;

use crate::entrypoint::NAME;

pub struct MyApp {
    // renderer: Option<RendererImpl>,
    element_store: ElementStore,
    physics: Physics,
}

impl Default for MyApp {
    fn default() -> Self {
        Self::new()
    }
}

impl MyApp {
    pub fn new() -> Self {
        Self {
            // renderer: None,
            element_store: ElementStore::new(),
            physics: Physics::new(),
            // world: World::new(),
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
        // self.renderer = Some(RenderImpl::new(
        //     config.format,
        //     Vector2::new(config.width, config.height),
        //     device,
        //     queue,
        //     NAME,
        // ));

        // if self.world.model_store().is_empty() {
        //     self.on_startup().await;
        // }
    }

    async fn on_suspend(&mut self) {
        // self.renderer = None;
    }

    async fn on_resize(&mut self, new_size: Vector2<u32>, device: &Device, queue: &Queue)
    where
        Self: Sized,
    {
        // if let Some(renderer) = &mut self.renderer {
        //     renderer.change_resolution(new_size, device, queue).await;
        // } else {
        //     warn!("Received resize event, but Renderer doesn't exist (yet?)");
        // }
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
        // TODO: New approach:
        // 1. ✅ ElementStore updates Elements and returns WorldChanges (Events).
        // 2. ❌ Events get separated here into whatever categories are needed.
        //    Events will be sent to each System as a mutable reference.
        //    Each system can remove the events it processed, unless they are universal.
        // 3. ✅ Update PhysicsSystem with Events and generate ChangeList.
        //    ⚠️ PhysicsSystem is not yet implemented, a dummy system will be implemented.
        // 4. Update Renderer with remaining Events + ChangeList.
        // 5. Return any AppEvents to the AppRuntime to be processed.
        // (6. Actually start rendering the frame after all events have been processed.)
        // ---
        // Anything "transformable" goes into the PhysicsSystem.
        // No matter if it can change or not (static).
        //
        // The PhysicsSystem will replace the World.
        // Anything World related, like WorldEnvironment, goes into the Renderer.

        let events = self.element_store.update(delta_time, input_state).await;

        let mut physics_events = Vec::<PhysicsEvent>::new();
        let mut element_events = Vec::<ElementEvent>::new(); // TODO: USE 
        let mut runtime_events = Vec::<RuntimeEvent>::new();

        for event in events {
            match event {
                Event::Model(model_event) => physics_events.push(PhysicsEvent::Model(model_event)),
                Event::Camera(camera_event) => {
                    physics_events.push(PhysicsEvent::Camera(camera_event))
                }
                Event::Element(element_event) => element_events.push(element_event),
                Event::App(runtime_event) => runtime_events.push(runtime_event), // TODO Should be Runtime not App Events
                Event::File(file_event) => todo!(),
                Event::Clear => physics_events.push(PhysicsEvent::Clear),
            }
        }

        let change_list = self.physics.update(delta_time, physics_events).await;

        // TODO: Sort events into buckets/categories.
        // TODO: Call each system async with the events.
        // TODO: "All" meaning Physics + File + Element
        // TODO: Renderer e.g. only needs ChangeList and needs to happen after the PhysicsSystem

        // TODO: Messages get created by Elements as WorldChanges.
        // World then transforms Message into AppChange.
        // Calling World::update returns any AppChanges.
        // We then return any AppChanges to the AppRuntime (if there are any).
        // The AppRuntime processes changes and puts any AppMessages into a queue.
        // And finally on the next update cycle we get the actual AppMessages.
        // I.e. the message already has been here, we just didn't process it and send it on.

        (!runtime_events.is_empty()).then_some(runtime_events)
    }

    async fn on_render(&mut self, target_view: &TextureView, device: &Device, queue: &Queue)
    where
        Self: Sized,
    {
        // self.world.prepare_render(device, queue);

        // if let Some(renderer) = &mut self.renderer {
        //     renderer
        //         .render(target_view, device, queue, &self.world)
        //         .await;
        // }
    }
}

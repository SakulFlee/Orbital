use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages,
};

use crate::engine::Engine;

use self::{
    object::Object,
    renderable::Renderable,
    updateable::{UpdateFrequency, Updateable},
};

pub mod object;
pub mod objects;
pub mod renderable;
pub mod updateable;

pub struct AppWorld {
    objects: Vec<Box<dyn Object>>,
    only_updateable: Vec<Box<dyn Updateable>>,
    only_renderable: Vec<Box<dyn Renderable>>,
}

impl AppWorld {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            only_updateable: Vec::new(),
            only_renderable: Vec::new(),
        }
    }

    pub fn spawn_object(&mut self, object: Box<dyn Object>) {
        self.objects.push(object);
    }

    pub fn spawn_updateable(&mut self, updateable: Box<dyn Updateable>) {
        self.only_updateable.push(updateable);
    }

    pub fn spawn_renderable(&mut self, renderable: Box<dyn Renderable>) {
        self.only_renderable.push(renderable);
    }

    /// Calls all registered updateables if their [`UpdateFrequency`]
    /// is set to [`UpdateFrequency::OnSecond`]
    pub fn call_updateables_on_second(&mut self, delta_time: f64) {
        // Update all `updateables` with `OnSecond` frequency
        self.only_updateable
            .iter_mut()
            .filter(|x| x.update_frequency() == UpdateFrequency::OnSecond)
            .for_each(|x| x.update(delta_time));

        // Update all `objects` with OnSecond frequency
        self.objects
            .iter_mut()
            .filter(|x| x.update_frequency() == UpdateFrequency::OnSecond)
            .for_each(|x| x.update(delta_time));
    }

    /// Calls all registered updateables if their [`UpdateFrequency`]
    /// is set to [`UpdateFrequency::OnCycle`]
    pub fn call_updateables_on_cycle(&mut self, delta_time: f64) {
        // Update all `updateables` with `OnCycle` frequency
        self.only_updateable
            .iter_mut()
            .filter(|x| x.update_frequency() == UpdateFrequency::OnCycle)
            .for_each(|x| x.update(delta_time));

        // Update all `objects` with OnCycle frequency
        self.objects
            .iter_mut()
            .filter(|x| x.update_frequency() == UpdateFrequency::OnCycle)
            .for_each(|x| x.update(delta_time));
    }

    pub fn call_renderables(&mut self, engine: &mut Engine) {
        // TODO: Fix for now ...
        if engine.has_vertex_buffer() {
            return;
        }

        // Process only renderable objects
        let mut vertex_buffers: Vec<(Buffer, u32)> = self
            // Retrieve vertices from Renderables
            .only_renderable
            .iter_mut()
            .filter(|x| x.do_render())
            .map(|x| x.vertices())
            .chain(
                // Retrieve vertices from Object
                self.objects
                    .iter_mut()
                    .filter(|x| x.do_render())
                    .map(|x| x.vertices()),
            )
            // Make Vertex Buffers
            .map(|x| {
                let number = x.len() as u32;
                (
                    engine
                        .get_device()
                        .create_buffer_init(&BufferInitDescriptor {
                            label: Some("Vertex Buffer"),
                            contents: bytemuck::cast_slice(x),
                            usage: BufferUsages::VERTEX,
                        }),
                    number,
                )
            })
            .collect();

        // TODO: Only takes the last buffer!
        if !vertex_buffers.is_empty() {
            let (buffer, number) = vertex_buffers.pop().expect("got no vertex buffers");
            engine.set_vertex_buffer(buffer, number);
        }
    }

    pub fn count_object(&self) -> usize {
        self.objects.iter().count()
    }

    pub fn count_updateable(&self) -> usize {
        self.only_updateable.iter().count()
    }

    pub fn count_renderable(&self) -> usize {
        self.only_renderable.iter().count()
    }
}

use std::sync::Arc;

use log::debug;
use winit::event::Event;
use winit::event_loop::EventLoopWindowTarget;
use winit::{event_loop::EventLoop, window::WindowBuilder};

use winit::window::Window as WinitWindow;

use crate::error::WindowError;

mod settings;
pub use settings::*;

pub struct Window {
    event_loop: Option<EventLoop<()>>,
    window: Arc<WinitWindow>,
}

impl Window {
    pub fn new(settings: WindowSettings) -> Result<Self, WindowError> {
        let event_loop = EventLoop::new().map_err(|e| WindowError::EventLoopError(e))?;

        let window = WindowBuilder::new()
            .with_inner_size(*settings.size())
            .build(&event_loop)
            .expect("Winit window/canvas creation failed");

        Ok(Self {
            event_loop: Some(event_loop),
            window: Arc::new(window),
        })
    }

    pub fn window(&self) -> Arc<WinitWindow> {
        self.window.clone()
    }

    pub fn run(&mut self) -> Result<(), WindowError> {
        self.event_loop
            .take()
            .expect("EventLoop disappeared!")
            .run(Self::event_handler)
            .map_err(|e| WindowError::EventLoopError(e))
    }

    fn event_handler(event: Event<()>, event_loop_window_target: &EventLoopWindowTarget<()>) {
        debug!("HIT");
    }
}

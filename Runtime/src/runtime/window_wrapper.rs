use std::sync::Arc;
use winit::{
    dpi::Size,
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

pub struct WindowWrapper {
    event_loop: EventLoop<()>,
    window: Arc<Window>,
}

impl WindowWrapper {
    pub fn new<S: Into<Size>>(event_loop: EventLoop<()>, title: &str, size: S) -> Self {
        let window = WindowBuilder::new()
            .with_title(title)
            .with_inner_size(size.into())
            .build(&event_loop)
            .unwrap();
        let window_arc = Arc::new(window);

        Self {
            event_loop,
            window: window_arc,
        }
    }

    pub fn window(&self) -> Arc<Window> {
        self.window.clone()
    }

    pub fn event_loop(self) -> EventLoop<()> {
        self.event_loop
    }
}

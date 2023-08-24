use winit::{
    dpi::PhysicalSize,
    event_loop::EventLoop,
    window::{Fullscreen, Window, WindowBuilder},
};

pub struct AppWindow {
    window: Window,
}

impl AppWindow {
    pub fn build_and_open(
        title: &str,
        size: PhysicalSize<u32>,
        maximized: bool,
        resizable: bool,
        fullscreen: Option<Fullscreen>,
        event_loop: &EventLoop<()>,
    ) -> Self {
        let mut builder = WindowBuilder::new();
        builder = builder.with_active(true);
        builder = builder.with_visible(true);
        builder = builder.with_title(title);
        builder = builder.with_inner_size(size);
        builder = builder.with_maximized(maximized);
        builder = builder.with_resizable(resizable);

        if fullscreen.is_some() {
            builder = builder.with_fullscreen(fullscreen);
        }

        let window = match builder.build(&event_loop) {
            Ok(window) => window,
            Err(err) => panic!("Window building failed! {:#?}", err),
        };

        Self { window }
    }

    pub fn get_window(&self) -> &Window {
        &self.window
    }
}

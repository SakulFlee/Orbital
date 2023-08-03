use winit::{
    dpi::{Position, Size},
    event_loop::EventLoop,
    window::{Fullscreen, Window as WWindow, WindowBuilder, WindowButtons},
};

pub struct Window {
    window: WWindow,
}

impl Window {
    pub fn build_and_open<S: Into<Size>>(
        title: &str,
        size: S,
        maximized: bool,
        resizable: bool,
        fullscreen: Option<Fullscreen>,
        position: Option<Position>,
        buttons: Option<WindowButtons>,
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

        if position.is_some() {
            builder = builder.with_position(position.unwrap());
        }

        if buttons.is_some() {
            builder = builder.with_enabled_buttons(buttons.unwrap());
        }

        let window = match builder.build(&event_loop) {
            Ok(window) => window,
            Err(err) => panic!("Window building failed! {:#?}", err),
        };

        Self { window }
    }

    pub fn get_window(&self) -> &WWindow {
        &self.window
    }
}

use std::{
    sync::Arc,
    thread::{spawn, JoinHandle},
};

use crate::{
    error::RuntimeError,
    gpu_connector::GPUConnector,
    window::{Window, WindowSettings},
};

use winit::window::Window as WinitWindow;

pub struct Runtime;

impl Runtime {
    pub fn liftoff() -> Result<(), RuntimeError> {
        // Spawn the window. This will already start opening and displaying!
        let mut window = Self::spawn_window()?;

        // Spawn any server components in a different thread
        let render_server_handle = Self::spawn_render_server(window.window());

        // Wake up the window and let it process things
        let _ = window.run().map_err(|e| RuntimeError::WindowError(e))?;

        // Once here: the window has closed!
        // This usually means our application should clean up and exit.
        // (or ... it crashed. Let's not hope it's that!)
        // Thus, join all threads and exit!
        render_server_handle
            .join()
            .expect("Render Server Join failed");

        Ok(())
    }

    fn spawn_window() -> Result<Window, RuntimeError> {
        let window_settings = WindowSettings::default();
        Window::new(window_settings).map_err(|e| RuntimeError::WindowError(e))
    }

    fn spawn_render_server(window: Arc<WinitWindow>) -> JoinHandle<()> {
        spawn(move || {
            let _connector = GPUConnector::new(Some(&window)).expect("GPU connector failure");
        })
    }
}

use std::sync::Arc;
use std::sync::Mutex;

use async_std::task::block_on;
use gilrs::Gilrs;
use log::trace;
use wgpu::TextureViewDescriptor;
use winit::{
    application::ApplicationHandler,
    error::EventLoopError,
    event::{DeviceEvent, DeviceId, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{CursorGrabMode, WindowId},
};

use super::{
    input::{InputEvent, InputState},
    Timer,
};
use super::{App, AppSettings};
use crate::{
    app::{AppContext, AppEvent, AppState},
    element::Message,
    logging::{self, debug, error, info, warn},
};

macro_rules! ctx_lock {
    ($ctx:ident) => {
        $ctx.lock().expect("Mutex failure")
    };
}

// TODO: State change

pub struct AppRuntime<AppImpl: App> {
    app: AppImpl,
    messages: Vec<Message>,
    settings: AppSettings,
    state: AppState,
    input_state: InputState,
    timer: Option<Timer>,
    #[cfg(feature = "gamepad_input")]
    gil: Gilrs,
}

impl<AppImpl: App> AppRuntime<AppImpl> {
    pub fn liftoff(
        event_loop: EventLoop<()>,
        settings: AppSettings,
        mut app: AppImpl,
    ) -> Result<(), EventLoopError> {
        logging::init();

        info!("Orbital Runtime");
        info!(" --- @SakulFlee --- ");

        block_on(app.on_startup());

        let mut app_runtime = Self {
            app,
            messages: Vec::new(),
            settings,
            state: AppState::Starting,
            input_state: InputState::new(),
            timer: None,
            #[cfg(feature = "gamepad_input")]
            gil: Gilrs::new().expect("Gamepad input initialization failed!"),
        };

        event_loop.run_app(&mut app_runtime)
    }

    pub fn redraw(&mut self) {
        let AppState::Ready(ctx) = &self.state else {
            error!(
                "Trying to redraw when app state is in a non-ready state! ({:?})",
                &self.state
            );
            return;
        };

        let lock = ctx_lock!(ctx);

        // Get next frame to render on
        let frame = match lock.acquire_next_frame() {
            Ok(surface_texture) => surface_texture,
            Err(e) => {
                warn!("Failed to acquire next frame from surface: {e}");
                return;
            }
        };

        let format = lock.get_first_view_format();

        let view: wgpu::TextureView = frame.texture.create_view(&TextureViewDescriptor {
            format: Some(format),
            ..TextureViewDescriptor::default()
        });

        block_on(self.app.on_render(&view, lock.device(), lock.queue()));

        frame.present();
    }

    #[cfg(feature = "gamepad_input")]
    fn receive_controller_inputs(&mut self) {
        use super::input::InputEvent;

        while let Some(gil_event) = self.gil.next_event() {
            if let Some(input_event) = InputEvent::convert_gil_event(gil_event) {
                self.input_state.handle_event(input_event);
            }
        }
    }

    fn update(&mut self) -> bool {
        let (delta_time, cycle) = self.timer.as_mut().expect("Timer went missing").tick();

        if let Some((total_delta, fps)) = cycle {
            debug!("FPS: {fps} | TDT: {total_delta}s | CDT: {delta_time}s");
        }

        // Check for gamepad input events if the feature is enabled
        #[cfg(feature = "gamepad_input_poll")]
        self.receive_controller_inputs();

        let result = if let Some(app_events) =
            block_on(self.app.on_update(&self.input_state, delta_time, cycle))
        {
            self.process_app_events(app_events)
        } else {
            false
        };

        self.input_state.reset_deltas();

        result
    }

    fn process_app_events(&mut self, app_events: Vec<AppEvent>) -> bool {
        let ctx = match &self.state {
            AppState::Ready(ctx) => ctx.clone(),
            _ => {
                debug!(
                    "App in invalid state ({:?}), skipping window events!",
                    self.state
                );
                return false;
            }
        };

        let mut exit_requested = false;
        for event in app_events {
            match event {
                AppEvent::ChangeCursorAppearance(cursor) => {
                    ctx_lock!(ctx).window().set_cursor(cursor);
                }
                AppEvent::ChangeCursorPosition(position) => {
                    if let Err(e) = ctx_lock!(ctx).window().set_cursor_position(position) {
                        error!("Failed to set cursor position: {e}");
                    }
                }
                AppEvent::ChangeCursorVisible(visible) => {
                    ctx_lock!(ctx).window().set_cursor_visible(visible);
                }
                AppEvent::ChangeCursorGrabbed(grab) => {
                    let lock = ctx_lock!(ctx);

                    if grab {
                        if let Err(e) = lock.window().set_cursor_grab(CursorGrabMode::Confined) {
                            error!(
                                    "Failed to set cursor grab! This might not be supported on your platform. Error: {e}"
                                );
                        }
                    } else if let Err(e) = lock.window().set_cursor_grab(CursorGrabMode::None) {
                        error!("Failed to unset cursor grab! Error: {e}");
                    }
                }
                AppEvent::RequestAppClosure => {
                    warn!("App closure was requested!");
                    exit_requested = true;
                    // TODO: Required? Marking the event_loop as exiting + AppState change might be
                    // enough
                }
                AppEvent::ForceAppClosure { exit_code } => {
                    warn!("Force app closure was requested with exit code {exit_code}!");
                    std::process::exit(exit_code);
                }
                AppEvent::RequestRedraw => {
                    ctx_lock!(ctx).window().request_redraw();
                }
                AppEvent::SendMessage(message) => {
                    self.messages.push(message);
                }
            }
        }

        exit_requested
    }

    fn exit(&mut self, event_loop: &ActiveEventLoop) {
        // Signal the application to close without forcing immediate cleanup
        // This allows the event loop to shut down gracefully
        event_loop.exit();
    }
}

impl<AppImpl: App> ApplicationHandler for AppRuntime<AppImpl> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if !matches!(self.state, AppState::Starting) || !matches!(self.state, AppState::Paused) {
            debug!(
                "Attempting to resume while not in required state! (State: {:?})",
                self.state
            );
        }

        debug!("Resuming app ...");

        let ctx = match AppContext::new(event_loop, &self.settings) {
            Ok(ctx) => ctx,
            Err(e) => {
                error!("Critical error: Failed to acquire context while resuming app!");
                trace!("Error: {:?}", e);
                return;
            }
        };

        self.state = AppState::Ready(Arc::new(Mutex::new(ctx)));

        self.timer = Some(Timer::new());

        info!("App resumed.");
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        if !matches!(self.state, AppState::Ready { .. }) {
            debug!(
                "Attempting to suspend while not in ready state! (State: {:?})",
                self.state
            );
        }

        debug!("Calling app.on_suspend()...");
        block_on(self.app.on_suspend());
        debug!("App.on_suspend() completed.");

        self.state = AppState::Paused;
        info!("App suspended!");
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let ctx = match &self.state {
            AppState::Ready(ctx) => ctx.clone(),
            _ => {
                debug!(
                    "App in invalid state ({:?}), skipping window events!",
                    self.state
                );
                return;
            }
        };

        let input_event = match event {
            WindowEvent::CloseRequested => {
                info!("App shutdown requested!");

                self.exit(event_loop);
                return;
            }
            WindowEvent::RedrawRequested => {
                if self.update() {
                    info!("App shutdown requested!");

                    self.exit(event_loop);
                    return;
                }
                self.redraw();

                #[cfg(feature = "auto_request_redraw")]
                ctx_lock!(ctx).window().request_redraw();

                None
            }
            WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => Some(InputEvent::KeyboardButton {
                device_id,
                event,
                is_synthetic,
            }),
            WindowEvent::MouseInput {
                device_id,
                state,
                button,
            } => Some(InputEvent::MouseButton {
                device_id,
                state,
                button,
            }),
            WindowEvent::MouseWheel {
                device_id,
                delta,
                phase,
            } => Some(InputEvent::MouseWheel {
                device_id,
                delta,
                phase,
            }),
            WindowEvent::CursorMoved {
                device_id,
                position,
            } => Some(InputEvent::MouseMovedPosition {
                device_id,
                position,
            }),
            WindowEvent::Resized(new_size) => {
                let ctx_lock = ctx_lock!(ctx);
                let configuration =
                    ctx_lock.make_surface_configuration(self.settings.vsync_enabled);
                ctx_lock.reconfigure_surface(&configuration);

                self.input_state.surface_resize(new_size);

                None
            }
            _ => None,
        };

        if let Some(input_event) = input_event {
            self.input_state.handle_event(input_event);
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        device_id: DeviceId,
        device_event: DeviceEvent,
    ) {
        if !matches!(&self.state, AppState::Ready(_)) {
            debug!(
                "App in invalid state ({:?}), skipping device events!",
                &self.state
            );
            return;
        }

        if let Some(input_event) = InputEvent::convert_device_event(device_id, device_event) {
            self.input_state.handle_event(input_event);
        }
    }
}

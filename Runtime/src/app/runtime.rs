use std::mem::transmute;
use std::thread;

use async_std::task::block_on;
use cgmath::Vector2;
use gilrs::Gilrs;
use log::trace;
use wgpu::{
    Adapter, Backend, BackendOptions, Backends, CompositeAlphaMode, Device, DeviceDescriptor,
    DeviceType, ExperimentalFeatures, Features, Instance, InstanceDescriptor, InstanceFlags,
    Limits, MemoryBudgetThresholds, MemoryHints, PresentMode, Queue, Surface, SurfaceConfiguration,
    SurfaceError, SurfaceTexture, TextureUsages, TextureViewDescriptor, Trace,
};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    error::EventLoopError,
    event::{DeviceEvent, DeviceId, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{CursorGrabMode, Window, WindowId},
};

use super::{
    input::{InputEvent, InputState},
    Timer,
};
use super::{App, AppSettings};
use crate::{
    app::{AppContext, AppEvent, AppState},
    element::{Element, Message},
    logging::{self, debug, error, info, warn},
};

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
        // Check if surface and device are present
        if self.surface.is_none() || self.device.is_none() {
            warn!("Redraw requested, but runtime is in an incomplete state!");
            return;
        }

        // Get next frame to render on
        let frame = match self.acquire_next_frame() {
            Ok(surface_texture) => surface_texture,
            Err(e) => {
                warn!("Failed to acquire next frame from surface: {e}");

                warn!("Attempting reconfiguration ...");
                self.reconfigure_surface();

                match self.acquire_next_frame() {
                    Ok(surface_texture) => surface_texture,
                    Err(e) => panic!(
                        "Failed to acquire next frame from surface after reconfiguration! ({e:?})"
                    ),
                }
            }
        };

        let format = match self
            .surface_configuration
            .as_ref()
            .unwrap()
            .view_formats
            .first()
        {
            Some(format) => format,
            None => {
                warn!("No view formats available for surface!");

                warn!("Attempting reconfiguration ...");
                self.reconfigure_surface();

                match self
                    .surface_configuration
                    .as_ref()
                    .unwrap()
                    .view_formats
                    .first()
                {
                    Some(format) => format,
                    None => panic!("No view formats available for surface after reconfiguration!"),
                }
            }
        };

        let view: wgpu::TextureView = frame.texture.create_view(&TextureViewDescriptor {
            format: Some(*format),
            ..TextureViewDescriptor::default()
        });

        block_on(self.app.on_render(
            &view,
            self.device.as_ref().unwrap(),
            self.queue.as_ref().unwrap(),
        ));

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
        let mut exit_requested = false;

        for event in app_events {
            match event {
                AppEvent::ChangeCursorAppearance(cursor) => {
                    if let Some(window) = &self.window {
                        window.set_cursor(cursor);
                    } else {
                        warn!("Change cursor appearance requested, but window does not exist!");
                    }
                }
                AppEvent::ChangeCursorPosition(position) => {
                    if let Some(window) = &self.window {
                        if let Err(e) = window.set_cursor_position(position) {
                            error!("Failed to set cursor position: {e}");
                        }
                    } else {
                        warn!("Change cursor position requested, but window does not exist!");
                    }
                }
                AppEvent::ChangeCursorVisible(visible) => {
                    if let Some(window) = &self.window {
                        window.set_cursor_visible(visible);
                    } else {
                        warn!("Change cursor visibility requested, but window does not exist!");
                    }
                }
                AppEvent::ChangeCursorGrabbed(grab) => {
                    if let Some(window) = &self.window {
                        if grab {
                            if let Err(e) = window.set_cursor_grab(CursorGrabMode::Confined) {
                                error!(
                                    "Failed to set cursor grab! This might not be supported on your platform. Error: {e}"
                                );
                            }
                        } else if let Err(e) = window.set_cursor_grab(CursorGrabMode::None) {
                            error!("Failed to unset cursor grab! Error: {e}");
                        }
                    } else {
                        warn!("Change cursor grabbing requested, but window does not exist!");
                    }
                }
                AppEvent::RequestAppClosure => {
                    warn!("App closure was requested!");
                    exit_requested = true;
                }
                AppEvent::ForceAppClosure { exit_code } => {
                    warn!("Force app closure was requested with exit code {exit_code}!");
                    std::process::exit(exit_code);
                }
                AppEvent::RequestRedraw => {
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    } else {
                        warn!("Redraw requested, but window does not exist!");
                    }
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

        self.state = AppState::Ready(ctx);

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
        let AppState::Ready(ctx) = self.state else {
            debug!(
                "App in invalid state, skipping window events! (State: {:?})",
                self.state
            );
            return;
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
                ctx.window().request_redraw();

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
                self.surface_configuration =
                    Some(AppRuntime::<AppImpl>::make_surface_configuration(
                        self.surface.as_ref().unwrap(),
                        self.adapter.as_ref().unwrap(),
                        new_size,
                        self.settings.vsync_enabled,
                    ));

                self.reconfigure_surface();

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
        if let Some(input_event) = InputEvent::convert_device_event(device_id, device_event) {
            self.input_state.handle_event(input_event);
        }
    }
}

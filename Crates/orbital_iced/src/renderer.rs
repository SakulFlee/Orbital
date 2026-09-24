use crate::state::IcedState;
use orbital_app::RenderLayer;
use orbital_app::render_overlay::{LayerRenderer as LayerRendererTrait, RenderOverlayContext};
use orbital_ecs_bridge::{
    AdapterResource, DeviceResource, IcedCapturedMouseDrag, IcedCapturedTouches, IcedEventQueue,
    IcedWindowEvent, QueueResource, SurfaceFormatResource, WindowSize,
};
use std::sync::Mutex;
use winit::event::{ElementState, MouseButton, TouchPhase};

pub struct IcedLayerRenderer {
    state: IcedState,
    inner: Mutex<Option<RendererInner>>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
}

struct RendererInner {
    renderer: iced_wgpu::Renderer,
    cache: Option<iced_runtime::user_interface::Cache>,
}

unsafe impl Send for IcedLayerRenderer {}
unsafe impl Sync for IcedLayerRenderer {}

impl IcedLayerRenderer {
    pub fn new(state: IcedState) -> Self {
        Self {
            state,
            inner: Mutex::new(None),
            device: None,
            queue: None,
        }
    }

    pub fn state(&self) -> &IcedState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut IcedState {
        &mut self.state
    }
}

impl LayerRendererTrait for IcedLayerRenderer {
    fn layer(&self) -> RenderLayer {
        RenderLayer::UI
    }

    /// Process input events and update `IcedCapturedTouches` **without**
    /// GPU rendering.  Called from `ModuleRuntime::update()` before game
    /// systems so touch-capture information is current when game touch
    /// input is processed.
    fn process_events(&mut self, ecs: &mut orbital_ecs::World) {
        // Ensure renderer is initialized (needed for text layout in hit-testing).
        let format = ecs
            .get_resource::<SurfaceFormatResource>()
            .map(|f| f.0)
            .unwrap_or(wgpu::TextureFormat::Bgra8UnormSrgb);

        // Clone device/queue from ECS resources on first call.
        if self.device.is_none() {
            self.device = ecs.get_resource::<DeviceResource>().map(|d| (*d.0).clone());
            self.queue = ecs.get_resource::<QueueResource>().map(|q| (*q.0).clone());
        }

        let device = match self.device {
            Some(ref d) => d,
            None => {
                log::warn!("No DeviceResource - skipping iced process_events");
                return;
            }
        };
        let queue = match self.queue {
            Some(ref q) => q,
            None => {
                log::warn!("No QueueResource - skipping iced process_events");
                return;
            }
        };

        {
            let mut guard = self.inner.lock().unwrap();
            if guard.is_none() {
                let adapter = match ecs.get_resource::<AdapterResource>() {
                    Some(a) => a.0.as_ref().clone(),
                    None => {
                        log::warn!("No AdapterResource - skipping iced process_events");
                        return;
                    }
                };
                let engine = iced_wgpu::Engine::new(
                    &adapter,
                    device.clone(),
                    queue.clone(),
                    format,
                    None,
                    iced_graphics::Shell::headless(),
                );
                let renderer = iced_wgpu::Renderer::new(engine, Default::default());
                *guard = Some(RendererInner {
                    renderer,
                    cache: Some(iced_runtime::user_interface::Cache::new()),
                });
            }
        }

        // Clone events from the shared queue (don't drain — render() also
        // clones, and the main runtime drains once after all renderers).
        let (iced_events, cursor_phys, modifiers, scale_factor) = {
            let queue = ecs.get_resource::<IcedEventQueue>();
            if let Some(ref q) = queue {
                (
                    q.events.clone(),
                    q.cursor_position,
                    q.modifiers,
                    q.scale_factor,
                )
            } else {
                (
                    Vec::new(),
                    None,
                    winit::keyboard::ModifiersState::empty(),
                    1.0,
                )
            }
        };

        let scale_factor = if scale_factor > 0.0 {
            scale_factor
        } else {
            1.0
        };

        // Physical → logical cursor position.
        let cursor = match cursor_phys {
            Some(pos) => iced_winit::core::mouse::Cursor::Available(iced_core::Point::new(
                (pos.x / scale_factor) as f32,
                (pos.y / scale_factor) as f32,
            )),
            None => iced_winit::core::mouse::Cursor::Unavailable,
        };

        // Convert events, tracking touch IDs for capture status mapping.
        #[derive(Clone, Copy)]
        enum MouseDragAction {
            Press,
            Release,
        }
        let mut iced_core_events = Vec::new();
        let mut event_touch_ids: Vec<Option<u64>> = Vec::new();
        let mut touch_release_info: Vec<(u64, TouchPhase)> = Vec::new();
        // Parallel to `iced_core_events`: primary-button press/release
        // events whose status decides `IcedCapturedMouseDrag`.
        let mut mouse_drag_events: Vec<Option<MouseDragAction>> = Vec::new();
        for evt in &iced_events {
            if let Some(converted) = convert_event(evt, &modifiers, scale_factor) {
                match evt {
                    IcedWindowEvent::Touch(touch) => {
                        event_touch_ids.push(Some(touch.id));
                        mouse_drag_events.push(None);
                        if matches!(touch.phase, TouchPhase::Ended | TouchPhase::Cancelled) {
                            touch_release_info.push((touch.id, touch.phase));
                        }
                    }
                    IcedWindowEvent::MouseInput { state, button }
                        if *button == MouseButton::Left =>
                    {
                        mouse_drag_events.push(Some(match state {
                            ElementState::Pressed => MouseDragAction::Press,
                            ElementState::Released => MouseDragAction::Release,
                        }));
                        event_touch_ids.push(None);
                    }
                    _ => {
                        event_touch_ids.push(None);
                        mouse_drag_events.push(None);
                    }
                }
                iced_core_events.push(converted);
            }
        }

        // Build view and interface.
        let view = self.state.view(ecs);
        let logical_size = iced_core::Size::new(
            ecs.get_resource::<WindowSize>()
                .map(|s| s.0.x as f32 / scale_factor as f32)
                .unwrap_or(800.0),
            ecs.get_resource::<WindowSize>()
                .map(|s| s.0.y as f32 / scale_factor as f32)
                .unwrap_or(600.0),
        );

        let mut guard = self.inner.lock().unwrap();
        let inner = guard.as_mut().unwrap();
        let renderer = &mut inner.renderer;

        let mut interface = iced_runtime::user_interface::UserInterface::build(
            view,
            logical_size,
            inner.cache.take().unwrap_or_default(),
            renderer,
        );

        let waker = iced_winit::core::shell::Waker::noop();
        let mut bus = iced_winit::core::shell::Bus::new();

        let (_, event_statuses) = interface.update(
            &NoopWindow,
            &waker,
            &iced_core_events,
            cursor,
            renderer,
            &mut bus,
        );

        // Process messages.
        for message in bus {
            self.state.handle_message(message);
        }

        // Update IcedCapturedTouches based on per-event capture statuses.
        for (status, touch_id) in event_statuses.iter().zip(event_touch_ids.iter()) {
            let Some(touch_id) = touch_id else {
                continue;
            };
            if *status == iced_winit::core::event::Status::Captured
                && let Some(mut captured) = ecs.get_resource_mut::<IcedCapturedTouches>()
                && captured.capture(*touch_id)
            {
                log::info!("iced: touch {touch_id} newly captured by the UI");
            }
        }
        for (touch_id, _phase) in &touch_release_info {
            if let Some(mut captured) = ecs.get_resource_mut::<IcedCapturedTouches>() {
                captured.release(*touch_id);
            }
        }

        // Primary-button presses that iced captured start a UI-owned
        // look-drag; releases end it. Captures union across renderers
        // (each press that any renderer captures wins), releases clear.
        for (status, action) in event_statuses.iter().zip(mouse_drag_events.iter()) {
            let Some(action) = action else {
                continue;
            };
            match action {
                MouseDragAction::Press => {
                    if *status == iced_winit::core::event::Status::Captured
                        && let Some(mut captured) = ecs.get_resource_mut::<IcedCapturedMouseDrag>()
                        && !captured.0
                    {
                        captured.capture();
                        log::info!("iced: mouse look-drag captured by the UI");
                    }
                }
                MouseDragAction::Release => {
                    if let Some(mut captured) = ecs.get_resource_mut::<IcedCapturedMouseDrag>() {
                        captured.release();
                    }
                }
            }
        }

        // Save updated cache.
        inner.cache = Some(interface.into_cache());
    }

    fn render(&mut self, ctx: RenderOverlayContext) {
        let format = ctx
            .ecs
            .get_resource::<SurfaceFormatResource>()
            .map(|f| f.0)
            .unwrap_or(wgpu::TextureFormat::Bgra8UnormSrgb);

        // Ensure renderer is initialized
        {
            let mut guard = self.inner.lock().unwrap();
            if guard.is_none() {
                let adapter = match ctx.ecs.get_resource::<AdapterResource>() {
                    Some(a) => a.0.as_ref().clone(),
                    None => {
                        log::warn!("No AdapterResource - skipping iced render");
                        return;
                    }
                };

                let engine = iced_wgpu::Engine::new(
                    &adapter,
                    ctx.device.clone(),
                    ctx.queue.clone(),
                    format,
                    None,
                    iced_graphics::Shell::headless(),
                );

                let renderer = iced_wgpu::Renderer::new(engine, Default::default());
                *guard = Some(RendererInner {
                    renderer,
                    cache: Some(iced_runtime::user_interface::Cache::new()),
                });
            }
        }

        // Read events from the ECS queue (clone, don't drain — multiple
        // IcedLayerRenderers share the same queue; the main runtime drains
        // once after all renderers have processed).
        //
        // Coordinate spaces: winit reports cursor/events/window size in
        // PHYSICAL pixels, while iced layout + hit-testing work in LOGICAL
        // pixels (physical / scale_factor). Mirror the canonical iced
        // `integration` example: convert cursor, CursorMoved events and the
        // layout size to logical, and build the viewport with the real
        // window scale so rendering matches. Skipping this makes the UI
        // offset grow with distance from the top-left on HiDPI displays.
        let (iced_events, cursor_phys, modifiers, scale_factor) = {
            let queue = ctx.ecs.get_resource::<IcedEventQueue>();
            if let Some(ref q) = queue {
                let events = q.events.clone();
                let cursor_pos = q.cursor_position;
                let mods = q.modifiers;
                let scale = q.scale_factor;
                (events, cursor_pos, mods, scale)
            } else {
                (
                    Vec::new(),
                    None,
                    winit::keyboard::ModifiersState::empty(),
                    1.0,
                )
            }
        };

        // Guard against a bogus scale (queue default is 1.0; scale should
        // never be <= 0 — fall back to 1.0 so we never divide by zero).
        let scale_factor = if scale_factor > 0.0 {
            scale_factor
        } else {
            1.0
        };

        // Physical → logical: iced layout + hit-testing expect logical
        // coordinates (same conversion as `conversion::cursor_position`).
        let cursor = match cursor_phys {
            Some(pos) => iced_winit::core::mouse::Cursor::Available(iced_core::Point::new(
                (pos.x / scale_factor) as f32,
                (pos.y / scale_factor) as f32,
            )),
            None => iced_winit::core::mouse::Cursor::Unavailable,
        };

        // Convert our owned events to iced events, remembering which winit
        // touch id each converted event belongs to (None for non-touch
        // events). Needed afterwards to map per-event capture statuses back
        // to winit touch ids for `IcedCapturedTouches`.
        let mut iced_core_events = Vec::new();
        let mut event_touch_ids: Vec<Option<u64>> = Vec::new();
        let mut touch_release_info: Vec<(u64, TouchPhase)> = Vec::new();
        for evt in &iced_events {
            if let Some(converted) = convert_event(evt, &modifiers, scale_factor) {
                match evt {
                    IcedWindowEvent::Touch(touch) => {
                        event_touch_ids.push(Some(touch.id));
                        if matches!(touch.phase, TouchPhase::Ended | TouchPhase::Cancelled) {
                            touch_release_info.push((touch.id, touch.phase));
                        }
                    }
                    _ => event_touch_ids.push(None),
                }
                iced_core_events.push(converted);
            }
        }

        // Build the view (borrows self.state temporarily).
        // `screen_size` is physical (winit `inner_size`); iced expects the
        // logical size here (= viewport.logical_size()).
        let view = self.state.view(ctx.ecs);
        let logical_size = iced_core::Size::new(
            ctx.screen_size.0 / scale_factor as f32,
            ctx.screen_size.1 / scale_factor as f32,
        );

        let mut guard = self.inner.lock().unwrap();
        let inner = guard.as_mut().unwrap();
        let renderer = &mut inner.renderer;

        let mut interface = iced_runtime::user_interface::UserInterface::build(
            view,
            logical_size,
            inner.cache.take().unwrap_or_default(),
            renderer,
        );

        let waker = iced_winit::core::shell::Waker::noop();
        let mut bus = iced_winit::core::shell::Bus::new();

        let (_, event_statuses) = interface.update(
            &NoopWindow,
            &waker,
            &iced_core_events,
            cursor,
            renderer,
            &mut bus,
        );

        interface.draw(
            renderer,
            &iced_winit::core::Theme::Dark,
            &iced_winit::core::renderer::Style::default(),
            cursor,
        );

        // Save the updated cache for next frame (into_cache consumes interface)
        let updated_cache = interface.into_cache();

        // Process messages
        for message in bus {
            self.state.handle_message(message);
        }

        // Track which fingers the UI consumed (`event::Status::Captured` —
        // e.g. button presses, `FloatingPanel` title-bar drags, sliders).
        // `module_runtime` consults this when feeding touch events into the
        // game-input path so UI touches don't also drive the virtual
        // joystick / drag-to-look camera.
        //
        // The release sweep runs even when this panel saw no lifted-touch
        // event, so stale ids can never linger and permanently block game
        // controls for that finger id.
        for (status, touch_id) in event_statuses.iter().zip(event_touch_ids.iter()) {
            let Some(touch_id) = touch_id else {
                continue;
            };
            if *status == iced_winit::core::event::Status::Captured
                && let Some(mut captured) = ctx.ecs.get_resource_mut::<IcedCapturedTouches>()
                && captured.capture(*touch_id)
            {
                log::info!("iced: touch {touch_id} newly captured by the UI");
            }
        }
        for (touch_id, _phase) in &touch_release_info {
            if let Some(mut captured) = ctx.ecs.get_resource_mut::<IcedCapturedTouches>() {
                captured.release(*touch_id);
            }
        }

        // Write back the cache after releasing the borrow on self.state
        inner.cache = Some(updated_cache);

        let physical_size =
            iced_core::Size::new(ctx.screen_size.0 as u32, ctx.screen_size.1 as u32);

        let viewport = iced_graphics::Viewport::with_physical_size(
            physical_size,
            iced_winit::core::renderer::Scale {
                window: scale_factor as f32,
                application: 1.0,
            },
        );

        inner
            .renderer
            .present(None, format, ctx.target_view, &viewport);
    }
}

fn convert_event(
    evt: &IcedWindowEvent,
    mods: &winit::keyboard::ModifiersState,
    scale_factor: f64,
) -> Option<iced_core::Event> {
    use iced_core::event::Event;
    use iced_core::keyboard;
    use iced_core::mouse;
    use iced_core::window;

    match evt {
        IcedWindowEvent::CursorMoved { position } => {
            // winit reports physical pixels; iced expects logical.
            Some(Event::Mouse(mouse::Event::CursorMoved {
                position: iced_core::Point::new(
                    (position.x / scale_factor) as f32,
                    (position.y / scale_factor) as f32,
                ),
            }))
        }
        IcedWindowEvent::MouseInput { state, button } => {
            let iced_button = match button {
                winit::event::MouseButton::Left => mouse::Button::Left,
                winit::event::MouseButton::Right => mouse::Button::Right,
                winit::event::MouseButton::Middle => mouse::Button::Middle,
                _ => return None,
            };
            let iced_state = match state {
                winit::event::ElementState::Pressed => mouse::Event::ButtonPressed(iced_button),
                winit::event::ElementState::Released => mouse::Event::ButtonReleased(iced_button),
            };
            Some(Event::Mouse(iced_state))
        }
        IcedWindowEvent::KeyboardInput {
            event,
            is_synthetic,
        } if !is_synthetic => {
            // `modifier_supplement` is unavailable on wasm32, Android, and
            // iOS in winit; fall back to the logical key / plain text there
            // (same as `iced_winit::conversion`).
            #[cfg(not(any(
                target_arch = "wasm32",
                target_os = "android",
                target_os = "ios"
            )))]
            use winit::platform::modifier_supplement::KeyEventExtModifierSupplement;

            let winit::event::KeyEvent {
                state,
                location,
                logical_key,
                physical_key: winit_physical,
                repeat,
                ..
            } = event;

            #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
            let k = convert_winit_key(&event.key_without_modifiers());
            #[cfg(any(target_arch = "wasm32", target_os = "android", target_os = "ios"))]
            let k = convert_winit_key(logical_key);
            let modified_key = convert_winit_key(logical_key);
            let phys = convert_physical_key(*winit_physical);
            #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
            let text = event.text_with_all_modifiers().map(iced_core::SmolStr::new);
            #[cfg(any(target_arch = "wasm32", target_os = "android", target_os = "ios"))]
            let text = event.text.clone();

            let location = match location {
                winit::keyboard::KeyLocation::Standard => keyboard::Location::Standard,
                winit::keyboard::KeyLocation::Left => keyboard::Location::Left,
                winit::keyboard::KeyLocation::Right => keyboard::Location::Right,
                winit::keyboard::KeyLocation::Numpad => keyboard::Location::Numpad,
            };

            let iced_mods = convert_modifiers(*mods);

            Some(Event::Keyboard(match state {
                winit::event::ElementState::Pressed => keyboard::Event::KeyPressed {
                    key: k,
                    modified_key,
                    physical_key: phys,
                    location,
                    modifiers: iced_mods,
                    text,
                    repeat: *repeat,
                },
                winit::event::ElementState::Released => keyboard::Event::KeyReleased {
                    key: k,
                    modified_key,
                    physical_key: phys,
                    location,
                    modifiers: iced_mods,
                },
            }))
        }
        IcedWindowEvent::ModifiersChanged(mods) => Some(Event::Keyboard(
            keyboard::Event::ModifiersChanged(convert_modifiers(*mods)),
        )),
        IcedWindowEvent::Touch(touch) => Some(Event::Touch(iced_winit::conversion::touch_event(
            *touch,
            scale_factor as f32,
        ))),
        IcedWindowEvent::RedrawRequested => Some(Event::Window(window::Event::RedrawRequested(
            iced_core::time::Instant::now(),
        ))),
        _ => None,
    }
}

fn convert_modifiers(m: winit::keyboard::ModifiersState) -> iced_core::keyboard::Modifiers {
    let mut mods = iced_core::keyboard::Modifiers::empty();
    if m.shift_key() {
        mods |= iced_core::keyboard::Modifiers::SHIFT;
    }
    if m.control_key() {
        mods |= iced_core::keyboard::Modifiers::CTRL;
    }
    if m.alt_key() {
        mods |= iced_core::keyboard::Modifiers::ALT;
    }
    if m.super_key() {
        mods |= iced_core::keyboard::Modifiers::COMMAND;
    }
    mods
}

fn convert_winit_key(k: &winit::keyboard::Key) -> iced_core::keyboard::Key {
    match k {
        winit::keyboard::Key::Named(named) => {
            iced_core::keyboard::Key::Named(convert_named_key(*named))
        }
        winit::keyboard::Key::Character(c) => iced_core::keyboard::Key::Character(c.clone()),
        winit::keyboard::Key::Unidentified(_) => iced_core::keyboard::Key::Unidentified,
        winit::keyboard::Key::Dead(None) => iced_core::keyboard::Key::Unidentified,
        winit::keyboard::Key::Dead(Some(d)) => {
            iced_core::keyboard::Key::Character(iced_core::SmolStr::new(d.to_string()))
        }
    }
}

fn convert_named_key(n: winit::keyboard::NamedKey) -> iced_core::keyboard::key::Named {
    use winit::keyboard::NamedKey;
    match n {
        NamedKey::Enter => iced_core::keyboard::key::Named::Enter,
        NamedKey::Backspace => iced_core::keyboard::key::Named::Backspace,
        NamedKey::Tab => iced_core::keyboard::key::Named::Tab,
        NamedKey::Escape => iced_core::keyboard::key::Named::Escape,
        NamedKey::Space => iced_core::keyboard::key::Named::Space,
        NamedKey::ArrowUp => iced_core::keyboard::key::Named::ArrowUp,
        NamedKey::ArrowDown => iced_core::keyboard::key::Named::ArrowDown,
        NamedKey::ArrowLeft => iced_core::keyboard::key::Named::ArrowLeft,
        NamedKey::ArrowRight => iced_core::keyboard::key::Named::ArrowRight,
        NamedKey::Home => iced_core::keyboard::key::Named::Home,
        NamedKey::End => iced_core::keyboard::key::Named::End,
        NamedKey::PageUp => iced_core::keyboard::key::Named::PageUp,
        NamedKey::PageDown => iced_core::keyboard::key::Named::PageDown,
        NamedKey::Delete => iced_core::keyboard::key::Named::Delete,
        NamedKey::Insert => iced_core::keyboard::key::Named::Insert,
        NamedKey::F1 => iced_core::keyboard::key::Named::F1,
        NamedKey::F2 => iced_core::keyboard::key::Named::F2,
        NamedKey::F3 => iced_core::keyboard::key::Named::F3,
        NamedKey::F4 => iced_core::keyboard::key::Named::F4,
        NamedKey::F5 => iced_core::keyboard::key::Named::F5,
        NamedKey::F6 => iced_core::keyboard::key::Named::F6,
        NamedKey::F7 => iced_core::keyboard::key::Named::F7,
        NamedKey::F8 => iced_core::keyboard::key::Named::F8,
        NamedKey::F9 => iced_core::keyboard::key::Named::F9,
        NamedKey::F10 => iced_core::keyboard::key::Named::F10,
        NamedKey::F11 => iced_core::keyboard::key::Named::F11,
        NamedKey::F12 => iced_core::keyboard::key::Named::F12,
        NamedKey::Alt => iced_core::keyboard::key::Named::Alt,
        NamedKey::Control => iced_core::keyboard::key::Named::Control,
        NamedKey::Shift => iced_core::keyboard::key::Named::Shift,
        NamedKey::Super => iced_core::keyboard::key::Named::Super,
        NamedKey::Meta => iced_core::keyboard::key::Named::Meta,
        _ => iced_core::keyboard::key::Named::Accept,
    }
}

fn convert_physical_key(pk: winit::keyboard::PhysicalKey) -> iced_core::keyboard::key::Physical {
    match pk {
        winit::keyboard::PhysicalKey::Code(code) => {
            iced_core::keyboard::key::Physical::Code(convert_key_code(code))
        }
        winit::keyboard::PhysicalKey::Unidentified(_) => {
            iced_core::keyboard::key::Physical::Unidentified(
                iced_core::keyboard::key::NativeCode::Unidentified,
            )
        }
    }
}

fn convert_key_code(code: winit::keyboard::KeyCode) -> iced_core::keyboard::key::Code {
    use winit::keyboard::KeyCode;
    match code {
        KeyCode::Backquote => iced_core::keyboard::key::Code::Backquote,
        KeyCode::Backslash => iced_core::keyboard::key::Code::Backslash,
        KeyCode::BracketLeft => iced_core::keyboard::key::Code::BracketLeft,
        KeyCode::BracketRight => iced_core::keyboard::key::Code::BracketRight,
        KeyCode::Comma => iced_core::keyboard::key::Code::Comma,
        KeyCode::Digit0 => iced_core::keyboard::key::Code::Digit0,
        KeyCode::Digit1 => iced_core::keyboard::key::Code::Digit1,
        KeyCode::Digit2 => iced_core::keyboard::key::Code::Digit2,
        KeyCode::Digit3 => iced_core::keyboard::key::Code::Digit3,
        KeyCode::Digit4 => iced_core::keyboard::key::Code::Digit4,
        KeyCode::Digit5 => iced_core::keyboard::key::Code::Digit5,
        KeyCode::Digit6 => iced_core::keyboard::key::Code::Digit6,
        KeyCode::Digit7 => iced_core::keyboard::key::Code::Digit7,
        KeyCode::Digit8 => iced_core::keyboard::key::Code::Digit8,
        KeyCode::Digit9 => iced_core::keyboard::key::Code::Digit9,
        KeyCode::Equal => iced_core::keyboard::key::Code::Equal,
        KeyCode::IntlBackslash => iced_core::keyboard::key::Code::IntlBackslash,
        KeyCode::IntlRo => iced_core::keyboard::key::Code::IntlRo,
        KeyCode::IntlYen => iced_core::keyboard::key::Code::IntlYen,
        KeyCode::KeyA => iced_core::keyboard::key::Code::KeyA,
        KeyCode::KeyB => iced_core::keyboard::key::Code::KeyB,
        KeyCode::KeyC => iced_core::keyboard::key::Code::KeyC,
        KeyCode::KeyD => iced_core::keyboard::key::Code::KeyD,
        KeyCode::KeyE => iced_core::keyboard::key::Code::KeyE,
        KeyCode::KeyF => iced_core::keyboard::key::Code::KeyF,
        KeyCode::KeyG => iced_core::keyboard::key::Code::KeyG,
        KeyCode::KeyH => iced_core::keyboard::key::Code::KeyH,
        KeyCode::KeyI => iced_core::keyboard::key::Code::KeyI,
        KeyCode::KeyJ => iced_core::keyboard::key::Code::KeyJ,
        KeyCode::KeyK => iced_core::keyboard::key::Code::KeyK,
        KeyCode::KeyL => iced_core::keyboard::key::Code::KeyL,
        KeyCode::KeyM => iced_core::keyboard::key::Code::KeyM,
        KeyCode::KeyN => iced_core::keyboard::key::Code::KeyN,
        KeyCode::KeyO => iced_core::keyboard::key::Code::KeyO,
        KeyCode::KeyP => iced_core::keyboard::key::Code::KeyP,
        KeyCode::KeyQ => iced_core::keyboard::key::Code::KeyQ,
        KeyCode::KeyR => iced_core::keyboard::key::Code::KeyR,
        KeyCode::KeyS => iced_core::keyboard::key::Code::KeyS,
        KeyCode::KeyT => iced_core::keyboard::key::Code::KeyT,
        KeyCode::KeyU => iced_core::keyboard::key::Code::KeyU,
        KeyCode::KeyV => iced_core::keyboard::key::Code::KeyV,
        KeyCode::KeyW => iced_core::keyboard::key::Code::KeyW,
        KeyCode::KeyX => iced_core::keyboard::key::Code::KeyX,
        KeyCode::KeyY => iced_core::keyboard::key::Code::KeyY,
        KeyCode::KeyZ => iced_core::keyboard::key::Code::KeyZ,
        KeyCode::Minus => iced_core::keyboard::key::Code::Minus,
        KeyCode::Period => iced_core::keyboard::key::Code::Period,
        KeyCode::Quote => iced_core::keyboard::key::Code::Quote,
        KeyCode::Semicolon => iced_core::keyboard::key::Code::Semicolon,
        KeyCode::Slash => iced_core::keyboard::key::Code::Slash,
        KeyCode::AltLeft => iced_core::keyboard::key::Code::AltLeft,
        KeyCode::AltRight => iced_core::keyboard::key::Code::AltRight,
        KeyCode::Backspace => iced_core::keyboard::key::Code::Backspace,
        KeyCode::CapsLock => iced_core::keyboard::key::Code::CapsLock,
        KeyCode::ContextMenu => iced_core::keyboard::key::Code::ContextMenu,
        KeyCode::ControlLeft => iced_core::keyboard::key::Code::ControlLeft,
        KeyCode::ControlRight => iced_core::keyboard::key::Code::ControlRight,
        KeyCode::Enter => iced_core::keyboard::key::Code::Enter,
        KeyCode::SuperLeft => iced_core::keyboard::key::Code::SuperLeft,
        KeyCode::SuperRight => iced_core::keyboard::key::Code::SuperRight,
        KeyCode::ShiftLeft => iced_core::keyboard::key::Code::ShiftLeft,
        KeyCode::ShiftRight => iced_core::keyboard::key::Code::ShiftRight,
        KeyCode::Space => iced_core::keyboard::key::Code::Space,
        KeyCode::Tab => iced_core::keyboard::key::Code::Tab,
        KeyCode::Delete => iced_core::keyboard::key::Code::Delete,
        KeyCode::End => iced_core::keyboard::key::Code::End,
        KeyCode::Home => iced_core::keyboard::key::Code::Home,
        KeyCode::Insert => iced_core::keyboard::key::Code::Insert,
        KeyCode::PageDown => iced_core::keyboard::key::Code::PageDown,
        KeyCode::PageUp => iced_core::keyboard::key::Code::PageUp,
        KeyCode::ArrowDown => iced_core::keyboard::key::Code::ArrowDown,
        KeyCode::ArrowLeft => iced_core::keyboard::key::Code::ArrowLeft,
        KeyCode::ArrowRight => iced_core::keyboard::key::Code::ArrowRight,
        KeyCode::ArrowUp => iced_core::keyboard::key::Code::ArrowUp,
        KeyCode::Escape => iced_core::keyboard::key::Code::Escape,
        KeyCode::F1 => iced_core::keyboard::key::Code::F1,
        KeyCode::F2 => iced_core::keyboard::key::Code::F2,
        KeyCode::F3 => iced_core::keyboard::key::Code::F3,
        KeyCode::F4 => iced_core::keyboard::key::Code::F4,
        KeyCode::F5 => iced_core::keyboard::key::Code::F5,
        KeyCode::F6 => iced_core::keyboard::key::Code::F6,
        KeyCode::F7 => iced_core::keyboard::key::Code::F7,
        KeyCode::F8 => iced_core::keyboard::key::Code::F8,
        KeyCode::F9 => iced_core::keyboard::key::Code::F9,
        KeyCode::F10 => iced_core::keyboard::key::Code::F10,
        KeyCode::F11 => iced_core::keyboard::key::Code::F11,
        KeyCode::F12 => iced_core::keyboard::key::Code::F12,
        _ => iced_core::keyboard::key::Code::Backquote,
    }
}

struct NoopWindow;

impl raw_window_handle::HasWindowHandle for NoopWindow {
    fn window_handle(
        &self,
    ) -> Result<raw_window_handle::WindowHandle<'static>, raw_window_handle::HandleError> {
        Err(raw_window_handle::HandleError::NotSupported)
    }
}

impl raw_window_handle::HasDisplayHandle for NoopWindow {
    fn display_handle(
        &self,
    ) -> Result<raw_window_handle::DisplayHandle<'static>, raw_window_handle::HandleError> {
        Err(raw_window_handle::HandleError::NotSupported)
    }
}

impl std::fmt::Debug for NoopWindow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NoopWindow").finish()
    }
}

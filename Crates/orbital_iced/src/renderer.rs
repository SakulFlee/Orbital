use crate::state::IcedState;
use orbital_app::render_overlay::{LayerRenderer as LayerRendererTrait, RenderOverlayContext};
use orbital_app::RenderLayer;
use orbital_ecs_bridge::{AdapterResource, IcedEventQueue, IcedWindowEvent, SurfaceFormatResource};
use std::sync::Mutex;

pub struct IcedLayerRenderer {
    state: IcedState,
    inner: Mutex<Option<RendererInner>>,
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

        // Consume events from the ECS queue
        let (iced_events, cursor, modifiers, scale_factor) = {
            let mut queue = ctx.ecs.get_resource_mut::<IcedEventQueue>();
            if let Some(ref mut q) = queue {
                let events = q.drain();
                let cursor_pos = q.cursor_position;
                let mods = q.modifiers;
                let scale = q.scale_factor;
                (events, cursor_pos, mods, scale)
            } else {
                (Vec::new(), None, winit::keyboard::ModifiersState::empty(), 1.0)
            }
        };

        // Convert cursor position from physical to logical coordinates
        let cursor = match cursor {
            Some(pos) => {
                let logical_x = pos.x / scale_factor;
                let logical_y = pos.y / scale_factor;
                iced_winit::core::mouse::Cursor::Available(iced_core::Point::new(
                    logical_x as f32,
                    logical_y as f32,
                ))
            }
            None => iced_winit::core::mouse::Cursor::Unavailable,
        };

        // Convert our owned events to iced events
        let mut iced_core_events = Vec::new();
        for evt in &iced_events {
            if let Some(converted) = convert_event(evt, &modifiers, scale_factor) {
                iced_core_events.push(converted);
            }
        }

        // Build the view (borrows self.state temporarily)
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

        let _ = interface.update(
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

        // Write back the cache after releasing the borrow on self.state
        inner.cache = Some(updated_cache);

        let physical_size = iced_core::Size::new(
            ctx.screen_size.0 as u32,
            ctx.screen_size.1 as u32,
        );

        let viewport = iced_graphics::Viewport::with_physical_size(
            physical_size,
            iced_winit::core::renderer::Scale {
                window: scale_factor as f32,
                application: scale_factor as f32,
            },
        );

        inner.renderer.present(None, format, ctx.target_view, &viewport);
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
            // Convert physical coordinates to logical
            let logical_x = position.x / scale_factor;
            let logical_y = position.y / scale_factor;
            Some(Event::Mouse(mouse::Event::CursorMoved {
                position: iced_core::Point::new(logical_x as f32, logical_y as f32),
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
        IcedWindowEvent::KeyboardInput { event, is_synthetic } if !is_synthetic => {
            use winit::platform::modifier_supplement::KeyEventExtModifierSupplement;

            let winit::event::KeyEvent {
                state,
                location,
                logical_key,
                physical_key: winit_physical,
                repeat,
                ..
            } = event;

            let k = convert_winit_key(&event.key_without_modifiers());
            let modified_key = convert_winit_key(logical_key);
            let phys = convert_physical_key(*winit_physical);
            let text = event
                .text_with_all_modifiers()
                .map(iced_core::SmolStr::new);

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
        IcedWindowEvent::ModifiersChanged(mods) => {
            Some(Event::Keyboard(keyboard::Event::ModifiersChanged(
                convert_modifiers(*mods),
            )))
        }
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

fn convert_physical_key(
    pk: winit::keyboard::PhysicalKey,
) -> iced_core::keyboard::key::Physical {
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

//! Viewport-aware layout selection for the debug overlay.
//!
//! The four floating windows were designed for a desktop-sized viewport: their
//! initial positions and sizes need roughly [`FLOATING_LAYOUT_MIN_WIDTH`] x
//! [`FLOATING_LAYOUT_MIN_HEIGHT`] logical pixels. On a phone, or on a small
//! desktop window, they cannot all fit.
//!
//! Instead of shrinking the windows until their text becomes unreadable, the
//! overlay switches to a tabbed layout: one full-screen panel at a time with a
//! tab strip for Performance, ECS, Console and Rendering. The floating windows
//! are still used whenever the viewport is large enough, and both layouts share
//! the same panel views and messages.

use iced_widget::{button, column, container, row, space, text};
use iced_winit::core::{Alignment, Background, Color, Element, Length, Theme};
use orbital_ecs::World;
use orbital_ecs_bridge::{IcedEventQueue, WindowSize};

use crate::panels::{blank, console_panel, ecs_panel, performance_panel, rendering_panel};
use crate::state::{DebugMessage, DebugUiState, PANEL_COUNT, PanelId, Renderer};

/// Smallest logical viewport width in which the four floating windows fit.
pub const FLOATING_LAYOUT_MIN_WIDTH: f32 = 1260.0;
/// Smallest logical viewport height in which the four floating windows fit.
pub const FLOATING_LAYOUT_MIN_HEIGHT: f32 = 880.0;

/// The viewport iced lays widgets out for, in logical pixels.
///
/// `WindowSize` is the physical window size; iced divides it by the current
/// scale factor. Returns `None` until the first `Resized` event arrives, when
/// the window size is still unknown.
pub fn logical_viewport(world: &World) -> Option<(f32, f32)> {
    let size = world.get_resource::<WindowSize>()?.0;
    if size.x == 0 || size.y == 0 {
        return None;
    }

    let scale = world
        .get_resource::<IcedEventQueue>()
        .map(|queue| queue.scale_factor)
        .filter(|scale| scale.is_finite() && *scale > 0.0)
        .unwrap_or(1.0);

    Some((size.x as f32 / scale as f32, size.y as f32 / scale as f32))
}

/// Whether the overlay should use the tabbed, one-panel-at-a-time layout.
///
/// This is driven by the viewport, not by the platform: a small desktop window
/// needs the same treatment as a phone in portrait, and a large tablet can keep
/// the floating windows.
pub fn uses_tabbed_layout(world: &World) -> bool {
    logical_viewport(world).is_some_and(|(width, height)| {
        width < FLOATING_LAYOUT_MIN_WIDTH || height < FLOATING_LAYOUT_MIN_HEIGHT
    })
}

/// Whether the tabbed overlay should be drawn.
pub fn tabbed_overlay_visible(world: &World) -> bool {
    uses_tabbed_layout(world)
        && world
            .get_resource::<DebugUiState>()
            .is_some_and(|state| state.visible)
}

/// The full-screen, tabbed overlay used when the floating windows do not fit.
///
/// One panel is shown at a time. The tab strip is always on screen, so every
/// panel can be opened again and the overlay can be hidden without a title bar
/// or a keyboard.
pub fn tabbed_overlay(world: &World) -> Element<'static, DebugMessage, Theme, Renderer> {
    let Some(state) = world.get_resource::<DebugUiState>() else {
        return blank();
    };

    if !state.visible {
        return blank();
    }

    let active = state.active_tab.min(PANEL_COUNT - 1);

    let mut tabs = row![].spacing(4);
    for index in 0..PANEL_COUNT {
        let Some(panel) = PanelId::from_index(index) else {
            continue;
        };

        let tab = button(text(panel.title()).size(12))
            .on_press(DebugMessage::SetTab(index))
            .padding([6, 8]);

        // The active tab uses the accent style so the current panel is obvious
        // without relying on a title bar.
        tabs = tabs.push(if index == active {
            tab.style(button::primary)
        } else {
            tab
        });
    }

    let hide = button(text("Hide").size(12))
        .on_press(DebugMessage::HideDebugUi)
        .padding([6, 10]);

    let header = container(
        row![tabs, space::horizontal(), hide]
            .spacing(8)
            .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .padding([6, 8])
    .style(tab_bar_style);

    let panel = PanelId::from_index(active).unwrap_or(PanelId::Performance);
    let body = match panel {
        PanelId::Performance => performance_panel(world),
        PanelId::Ecs => ecs_panel(world),
        PanelId::Console => console_panel(world),
        PanelId::Rendering => rendering_panel(world),
    };

    container(column![header, body])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(overlay_style)
        .into()
}

/// The panel background, matching the floating windows' palette.
///
/// Unlike a floating window there is no title bar and no gap around the panel,
/// so the background has to be opaque enough that the game render stays out of
/// the way of the controls.
fn overlay_style(_theme: &Theme) -> iced_widget::container::Style {
    iced_widget::container::Style {
        background: Some(Background::Color(Color::from_rgba(0.06, 0.06, 0.10, 0.96))),
        ..Default::default()
    }
}

/// The tab strip background, slightly lighter than the panel body.
fn tab_bar_style(_theme: &Theme) -> iced_widget::container::Style {
    iced_widget::container::Style {
        background: Some(Background::Color(Color::from_rgba(0.12, 0.12, 0.20, 1.0))),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cgmath::Vector2;

    fn world_with_size(width: u32, height: u32, scale_factor: f64) -> World {
        let mut world = World::new();
        world.insert_resource(WindowSize(Vector2::new(width, height)));
        let mut events = IcedEventQueue::default();
        events.set_scale_factor(scale_factor);
        world.insert_resource(events);
        world
    }

    #[test]
    fn a_desktop_window_keeps_the_floating_layout() {
        let world = world_with_size(1920, 1080, 1.0);

        assert_eq!(logical_viewport(&world), Some((1920.0, 1080.0)));
        assert!(!uses_tabbed_layout(&world));
    }

    #[test]
    fn a_phone_uses_the_tabbed_layout_in_both_orientations() {
        let portrait = world_with_size(1080, 2400, 2.75);
        let landscape = world_with_size(2400, 1080, 2.75);

        assert!(uses_tabbed_layout(&portrait));
        assert!(uses_tabbed_layout(&landscape));
    }

    #[test]
    fn a_large_display_with_scaling_can_still_be_compact() {
        // 1920x1080 physical at 200% is only 960x540 logical pixels.
        let world = world_with_size(1920, 1080, 2.0);

        assert_eq!(logical_viewport(&world), Some((960.0, 540.0)));
        assert!(uses_tabbed_layout(&world));
    }

    #[test]
    fn a_window_exactly_at_the_floating_minimum_keeps_the_windows() {
        let world = world_with_size(
            FLOATING_LAYOUT_MIN_WIDTH as u32,
            FLOATING_LAYOUT_MIN_HEIGHT as u32,
            1.0,
        );

        assert!(!uses_tabbed_layout(&world));
    }

    #[test]
    fn an_unknown_or_zero_viewport_does_not_switch_layouts() {
        let unknown = World::new();
        assert_eq!(logical_viewport(&unknown), None);
        assert!(!uses_tabbed_layout(&unknown));

        let zero = world_with_size(0, 0, 1.0);
        assert_eq!(logical_viewport(&zero), None);
        assert!(!uses_tabbed_layout(&zero));
    }

    #[test]
    fn the_tabbed_overlay_needs_an_initialized_state_and_a_small_viewport() {
        let mut world = world_with_size(1280, 720, 1.0);
        assert!(!tabbed_overlay_visible(&world), "no state resource yet");

        world.insert_resource(DebugUiState::new(winit::keyboard::KeyCode::F1));
        assert!(!tabbed_overlay_visible(&world), "the overlay starts hidden");

        world
            .get_resource_mut::<DebugUiState>()
            .expect("state was just inserted")
            .visible = true;
        assert!(tabbed_overlay_visible(&world));
    }
}

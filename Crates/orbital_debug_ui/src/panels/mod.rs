//! The individual debug panels.
//!
//! Each panel is a pure function from the ECS world to a widget tree, which is
//! what `orbital_iced` expects of `IcedState::with_view`.

pub mod console;
pub mod ecs;
pub mod edit;
pub mod performance;

pub use console::console_panel;
pub use ecs::ecs_panel;

pub use performance::performance_panel;

use std::any::TypeId;

use iced_widget::Column;
use iced_winit::core::{Color, Element, Length, Theme};

use orbital_ecs::{Entity, World};
use orbital_ecs_bridge::{CameraDescriptorEcs, ModelDescriptorEcs};

use crate::state::DebugMessage;
use crate::state::Renderer as IcedRenderer;

/// The view a panel renders when it is hidden or its state is missing.
///
/// Must fill the whole window: the runtime builds a full-window
/// `UserInterface` per panel, and a zero-sized tree would break hit-testing for
/// the panels drawn after it. Two widgets is also cheap enough that a hidden
/// panel costs almost nothing per frame.
pub(crate) fn blank<'a>() -> Element<'a, DebugMessage, Theme, IcedRenderer> {
    iced_widget::container(iced_widget::text(""))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// A horizontal rule, used to separate sections.
pub(crate) fn divider<'a>() -> Element<'a, DebugMessage, Theme, IcedRenderer> {
    iced_widget::container(iced_widget::text(""))
        .height(1)
        .width(Length::Fill)
        .into()
}

/// A small caption, e.g. a column heading.
pub(crate) fn caption<'a>(
    label: &str,
    color: Color,
) -> Element<'a, DebugMessage, Theme, IcedRenderer> {
    iced_widget::row![iced_widget::text(label.to_string()).size(12).color(color)].into()
}

/// Lays out already-collected rows.
///
/// `column!` only accepts a literal list, and every panel here builds its rows
/// in a loop under a budget, so the builder is assembled by hand.
pub(crate) fn column_of<'a>(
    rows: Vec<Element<'a, DebugMessage, Theme, IcedRenderer>>,
    spacing: f32,
) -> Column<'a, DebugMessage, Theme, IcedRenderer> {
    iced_widget::column(rows).spacing(spacing).padding(8)
}

/// A scrollable, full-size body, for panels that can overflow.
pub(crate) fn scrolling<'a>(
    content: impl Into<Element<'a, DebugMessage, Theme, IcedRenderer>>,
) -> Element<'a, DebugMessage, Theme, IcedRenderer> {
    iced_widget::scrollable(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// A human-readable label for `entity`, if the engine knows one.
///
/// Models and cameras carry their own labels; every other entity falls back to
/// its index alone. An empty label is treated as absent.
pub(crate) fn entity_label(world: &World, entity: &Entity) -> Option<String> {
    let model = world
        .component_debug(TypeId::of::<ModelDescriptorEcs>(), entity)
        .and_then(|handle| {
            handle
                .as_any()
                .downcast_ref::<ModelDescriptorEcs>()
                .map(|descriptor| descriptor.label.clone())
        });

    let camera = world
        .component_debug(TypeId::of::<CameraDescriptorEcs>(), entity)
        .and_then(|handle| {
            handle
                .as_any()
                .downcast_ref::<CameraDescriptorEcs>()
                .map(|descriptor| descriptor.label.clone())
        });

    model
        .into_iter()
        .chain(camera)
        .find(|label| !label.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entity_label_uses_a_camera_descriptor_label() {
        let mut world = World::new();
        let entity = world.spawn_entity();
        world
            .attach_component(
                &entity,
                CameraDescriptorEcs {
                    label: "Main camera".to_string(),
                    ..CameraDescriptorEcs::default()
                },
            )
            .expect("attachment failed");

        assert_eq!(
            entity_label(&world, &entity).as_deref(),
            Some("Main camera")
        );
    }

    #[test]
    fn entity_label_is_none_without_a_labelled_descriptor() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        assert_eq!(entity_label(&world, &entity), None);
    }
}

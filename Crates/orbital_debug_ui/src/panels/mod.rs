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
use orbital_ecs_bridge::{
    ActiveCamera, CameraDescriptor, LightDescriptor, ModelDescriptor, ModelInstances,
};

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
/// A human-readable role/label summary for `entity`, if the engine knows one.
///
/// The ECS row prepends `#{index}:`; this function only returns the role,
/// label and useful details such as `Camera - Default (active)` or
/// `Model - procgeo (1 inst, 15.0k v/30.0k i)`.
pub(crate) fn entity_label(world: &World, entity: &Entity) -> Option<String> {
    let active_camera = world.get_resource::<ActiveCamera>().map(|active| active.0);

    let camera = world
        .component_debug(TypeId::of::<CameraDescriptor>(), entity)
        .and_then(|handle| {
            handle
                .as_any()
                .downcast_ref::<CameraDescriptor>()
                .map(|descriptor| descriptor.label.clone())
        });

    let light = world
        .component_debug(TypeId::of::<LightDescriptor>(), entity)
        .and_then(|handle| {
            handle
                .as_any()
                .downcast_ref::<LightDescriptor>()
                .map(LightDescriptor::kind_label)
        });

    let model = world
        .component_debug(TypeId::of::<ModelDescriptor>(), entity)
        .and_then(|handle| {
            handle
                .as_any()
                .downcast_ref::<ModelDescriptor>()
                .map(|descriptor| {
                    let instances = world
                        .component_debug(TypeId::of::<ModelInstances>(), entity)
                        .and_then(|instances| {
                            instances
                                .as_any()
                                .downcast_ref::<ModelInstances>()
                                .map(|instances| instances.0.len())
                        })
                        .unwrap_or(0);

                    (
                        descriptor.label.clone(),
                        instances,
                        descriptor.mesh.vertices.len(),
                        descriptor.mesh.indices.len(),
                    )
                })
        });

    let mut roles = Vec::new();
    let mut label = None;
    let mut details = Vec::new();

    if let Some(camera) = camera {
        roles.push("Camera");
        if !camera.trim().is_empty() {
            label = Some(camera);
        }
        if active_camera == Some(*entity) {
            details.push("active".to_string());
        }
    }

    if let Some(light) = light {
        roles.push("Light");
        if label.is_none() {
            label = Some(light.to_string());
        } else {
            details.push(light.to_string());
        }
    }

    if let Some((model_label, instances, vertices, indices)) = model {
        roles.push("Model");
        if label.is_none() && !model_label.trim().is_empty() {
            label = Some(model_label);
        } else if !model_label.trim().is_empty() {
            details.push(model_label);
        }

        if instances > 0 {
            details.push(format!("{instances} inst"));
        }

        if vertices > 0 || indices > 0 {
            details.push(format!(
                "{}v/{}i",
                format_count(vertices),
                format_count(indices)
            ));
        }
    }

    if roles.is_empty() {
        return None;
    }

    let mut summary = roles.join("/");
    if let Some(label) = label {
        summary.push_str(" - ");
        summary.push_str(&label);
    }
    if !details.is_empty() {
        summary.push_str(" (");
        summary.push_str(&details.join(", "));
        summary.push(')');
    }

    Some(summary)
}

fn format_count(value: usize) -> String {
    if value >= 1_000 {
        format!("{:.1}k", value as f64 / 1_000.0)
    } else {
        value.to_string()
    }
}

/// A high-contrast text-field style for the debug panels.
///
/// The default iced input is a mid-gray box that reads as disabled against the
/// debug windows' darker background. This keeps the value bright and gives the
/// field a deliberate border, with an accent when focused.
pub(crate) fn input_style(
    theme: &Theme,
    status: iced_widget::text_input::Status,
) -> iced_widget::text_input::Style {
    let mut style = iced_widget::text_input::default(theme, status);

    style.value = Color::from_rgb(0.93, 0.95, 0.98);
    style.placeholder = Color::from_rgb(0.52, 0.56, 0.64);
    style.background =
        iced_widget::core::Background::Color(Color::from_rgba(0.10, 0.10, 0.16, 0.95));
    style.border = iced_widget::core::border::Border {
        color: Color::from_rgba(0.45, 0.5, 0.65, 0.65),
        width: 1.0,
        radius: 3.0.into(),
    };

    if matches!(status, iced_widget::text_input::Status::Focused { .. }) {
        style.border.color = Color::from_rgb(0.45, 0.7, 1.0);
    } else if matches!(status, iced_widget::text_input::Status::Hovered) {
        style.border.color = Color::from_rgba(0.65, 0.7, 0.85, 0.8);
    }

    style
}
#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use cgmath::Vector3;
    use orbital_mesh::MeshDescriptor;

    use super::*;

    #[test]
    fn entity_label_includes_the_camera_role_and_label() {
        let mut world = World::new();
        let entity = world.spawn_entity();
        world
            .attach_component(
                &entity,
                CameraDescriptor {
                    label: "Main camera".to_string(),
                    ..CameraDescriptor::default()
                },
            )
            .expect("attachment failed");

        assert_eq!(
            entity_label(&world, &entity).as_deref(),
            Some("Camera - Main camera")
        );
    }

    #[test]
    fn the_active_camera_is_marked() {
        let mut world = World::new();
        let entity = world.spawn_entity();
        world
            .attach_component(&entity, CameraDescriptor::default())
            .expect("attachment failed");
        world.insert_resource(ActiveCamera(entity));

        assert_eq!(
            entity_label(&world, &entity).as_deref(),
            Some("Camera - Default (active)")
        );
    }

    #[test]
    fn lights_show_their_subtype() {
        let mut world = World::new();
        let entity = world.spawn_entity();
        world
            .attach_component(
                &entity,
                LightDescriptor::new_spot(
                    Vector3::new(1.0, 1.0, 1.0),
                    1.0,
                    Vector3::new(0.0, -1.0, 0.0),
                    0.1,
                    0.5,
                ),
            )
            .expect("attachment failed");

        assert_eq!(
            entity_label(&world, &entity).as_deref(),
            Some("Light - Spot")
        );
    }

    #[test]
    fn models_show_their_instance_count() {
        let mut world = World::new();
        let entity = world.spawn_entity();
        world
            .attach_component(
                &entity,
                ModelDescriptor {
                    label: "procgeo".to_string(),
                    mesh: Arc::new(MeshDescriptor::new(Vec::new(), Vec::new())),
                    materials: Vec::new(),
                },
            )
            .expect("descriptor attach failed");
        let mut instances = ModelInstances::new();
        instances.add_identity_instance();
        world
            .attach_component(&entity, instances)
            .expect("instances attach failed");

        assert_eq!(
            entity_label(&world, &entity).as_deref(),
            Some("Model - procgeo (1 inst)")
        );
    }

    #[test]
    fn entity_label_is_none_without_a_recognised_descriptor() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        assert_eq!(entity_label(&world, &entity), None);
    }

    #[test]
    fn large_counts_are_abbreviated() {
        assert_eq!(format_count(999), "999");
        assert_eq!(format_count(1_500), "1.5k");
    }
}

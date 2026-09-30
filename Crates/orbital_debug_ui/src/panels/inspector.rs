//! Panel showing the selected entity's components, with inline editing.

use std::any::TypeId;

use iced_widget::{button, column, container, row, text, text_input};
use iced_winit::core::{Color, Element, Length, Theme};

use orbital_ecs::{Entity, World};

use crate::editable::{apply_to, display_for, editable_label, editable_ops};
use crate::state::Renderer as IcedRenderer;
use crate::state::{DebugMessage, DebugUiState, PanelId};

const PANEL_WIDTH: f32 = 320.0;

/// Builds the inspector for the selected entity.
///
/// Each editable component gets a text field seeded with its current value;
/// applying it runs the value back through the ECS. Components with no editor
/// are shown read-only, because there is no way to parse text back into a type
/// the engine knows nothing about.
pub fn inspector_panel<'a>(ecs: &World) -> Element<'a, DebugMessage, Theme, IcedRenderer> {
    let Some(state) = ecs.get_resource::<DebugUiState>() else {
        return crate::panels::blank();
    };

    if !state.is_panel_visible(PanelId::Inspector) {
        return crate::panels::blank();
    }

    let Some(entity) = state.selected else {
        return container(
            column![
                text("no entity selected").size(13),
                crate::panels::caption("pick one in the ECS panel", Color::from_rgb(0.6, 0.6, 0.6))
            ]
            .spacing(4)
            .padding(10),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into();
    };

    if !ecs.is_valid(&entity) {
        return container(
            text(format!("Entity {{ {} }} no longer exists", entity.index))
                .size(13)
                .color(Color::from_rgb(1.0, 0.6, 0.5)),
        )
        .padding(10)
        .into();
    }

    let mut rows = vec![
        crate::panels::caption(
            &format!("Entity {{ {} }}", entity.index),
            Color::from_rgb(0.7, 0.85, 1.0),
        ),
        crate::panels::divider(),
    ];

    let components = ecs.entity_components(&entity);
    if components.is_empty() {
        rows.push(row![text("this entity has no components").size(12)].into());
    }

    for (type_id, name) in components {
        rows.push(component_editor(ecs, &state, &entity, type_id, name));
    }

    if let Some(error) = state.last_error.as_deref() {
        rows.push(crate::panels::divider());
        rows.push(
            text(error.to_string())
                .size(12)
                .color(Color::from_rgb(1.0, 0.5, 0.45))
                .into(),
        );
    }

    container(
        column![crate::panels::scrolling(crate::panels::column_of(
            rows, 4.0
        ))]
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .width(PANEL_WIDTH)
    .into()
}

/// One component: a labelled value, editable when the type has an editor.
fn component_editor(
    ecs: &World,
    state: &DebugUiState,
    entity: &Entity,
    type_id: TypeId,
    name: &'static str,
) -> Element<'static, DebugMessage, Theme, IcedRenderer> {
    // Everything needed is taken as owned strings, so the resource guard is
    // released before the element tree is built.
    let debug_value = ecs
        .component_debug_string(type_id, entity)
        .map(|value| DebugUiState::preview(&value))
        .unwrap_or_else(|| "<unavailable>".to_string());

    // The editable text form, which is not the same as the `Debug` form:
    // `Position` prints as `Position(Point3 { x: .. })` but edits as "1 2 3".
    let edit_value = ecs
        .component_debug(type_id, entity)
        .and_then(|handle| display_for(type_id, handle.as_any()));

    let label = short_type_name(name);

    let Some(edit_value) = edit_value else {
        return column![
            text(label).size(12).color(Color::from_rgb(0.7, 0.85, 1.0)),
            text(debug_value).size(12),
            text("read-only")
                .size(10)
                .color(Color::from_rgb(0.55, 0.55, 0.6)),
        ]
        .spacing(0)
        .padding(2)
        .width(Length::Fill)
        .into();
    };

    let type_name = editable_label(type_id).unwrap_or("value").to_string();
    let buffer = state.field_buffer(type_id, &edit_value);
    let field = FieldKey {
        entity: *entity,
        type_id,
    };

    let input = text_input(type_name, buffer)
        .on_input(move |text| DebugMessage::EditField(field, text))
        .on_submit(DebugMessage::ApplyField(field))
        .width(Length::Fill)
        .size(12);

    let apply = button(text("set").size(11))
        .on_press(DebugMessage::ApplyField(field))
        .width(Length::Shrink);

    column![
        text(label).size(12).color(Color::from_rgb(0.7, 0.85, 1.0)),
        row![input, apply]
            .spacing(4)
            .align_y(iced_winit::core::Alignment::Center),
    ]
    .spacing(1)
    .padding(2)
    .width(Length::Fill)
    .into()
}

/// Identifies the field a message refers to.
///
/// `Copy` so a widget can hold it in two closures (`on_input` and `on_submit`)
/// without cloning.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FieldKey {
    pub entity: Entity,
    pub type_id: TypeId,
}

/// The last path segment of a type name.
fn short_type_name(name: &str) -> String {
    name.rsplit("::").next().unwrap_or(name).to_string()
}

/// Writes `input` into the component `key` refers to.
///
/// Reports failures instead of panicking, so a bad value leaves the component
/// untouched and the panel can show the message.
pub fn apply_edit(world: &World, key: FieldKey, input: &str) -> Result<(), String> {
    if !world.is_valid(&key.entity) {
        return Err(format!(
            "Entity {{ {} }} no longer exists",
            key.entity.index
        ));
    }

    editable_ops(key.type_id).ok_or_else(|| "this component type is not editable".to_string())?;

    let mut handle = world
        .component_mut_any(key.type_id, &key.entity)
        .ok_or_else(|| "the component is no longer attached".to_string())?;

    apply_to(key.type_id, handle.as_any_mut(), input)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world_with_position() -> (World, Entity) {
        let mut world = World::new();
        let entity = world.spawn_entity();
        world
            .attach_component(
                &entity,
                orbital_ecs_bridge::Position(cgmath::Point3::new(1.0, 2.0, 3.0)),
            )
            .expect("attachment failed");

        (world, entity)
    }

    #[test]
    fn a_valid_edit_is_applied() {
        let (world, entity) = world_with_position();
        let key = FieldKey {
            entity,
            type_id: TypeId::of::<orbital_ecs_bridge::Position>(),
        };

        apply_edit(&world, key, "9 8 7").expect("edit failed");

        assert_eq!(
            world.component_debug_string(key.type_id, &entity),
            Some("Position(Point3 [9.0, 8.0, 7.0])".to_string())
        );
    }

    #[test]
    fn a_bad_value_is_reported_and_the_component_is_untouched() {
        let (world, entity) = world_with_position();
        let key = FieldKey {
            entity,
            type_id: TypeId::of::<orbital_ecs_bridge::Position>(),
        };

        let error = apply_edit(&world, key, "1 2").expect_err("should fail");
        assert!(error.contains("expected 3 numbers"), "unexpected: {error}");

        let error = apply_edit(&world, key, "not a vector").expect_err("should fail");
        assert!(error.contains("is not a number"), "unexpected: {error}");

        assert_eq!(
            world.component_debug_string(key.type_id, &entity),
            Some("Position(Point3 [1.0, 2.0, 3.0])".to_string()),
            "the component is untouched after a failed edit"
        );
    }

    #[test]
    fn editing_a_non_editable_type_is_refused() {
        let (mut world, entity) = world_with_position();
        world
            .attach_component(&entity, orbital_ecs_bridge::CameraDirty(false))
            .expect("attachment failed");

        let key = FieldKey {
            entity,
            type_id: TypeId::of::<orbital_ecs_bridge::CameraDirty>(),
        };

        assert!(apply_edit(&world, key, "true").is_err());
    }

    #[test]
    fn editing_a_despawned_entity_is_refused() {
        let (mut world, entity) = world_with_position();
        let key = FieldKey {
            entity,
            type_id: TypeId::of::<orbital_ecs_bridge::Position>(),
        };

        world.despawn_entity(&entity);

        let error = apply_edit(&world, key, "1 1 1").expect_err("should fail");
        assert!(error.contains("no longer exists"), "unexpected: {error}");
    }

    #[test]
    fn short_type_name_strips_the_module_path() {
        assert_eq!(short_type_name("my_game::components::Position"), "Position");
        assert_eq!(short_type_name("Position"), "Position");
    }
}

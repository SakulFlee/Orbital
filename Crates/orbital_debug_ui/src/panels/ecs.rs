//! Panel listing live entities and editing the selected entity's components.

use std::any::TypeId;

use iced_widget::{button, column, container, row, text, text_input};
use iced_winit::core::text::Wrapping;
use iced_winit::core::{Color, Element, Font, Length, Theme};

use orbital_ecs::{Entity, World};
use orbital_ecs_bridge::{
    EditableField, EditableValue, MODEL_INSTANCE_EDIT_LIMIT, ModelInstances, euler_degrees_text,
    model_instance_field_id, short_model_instance_id,
};

use crate::editable::{display_for, editable_fields_ops, editable_label, is_editable};
use crate::panels::edit::{FieldKey, short_type_name};
use crate::state::Renderer as IcedRenderer;
use crate::state::{DebugMessage, DebugUiState, MAX_ROWS, PanelId};

/// How much of a read-only component value the panel shows.
const DETAIL_PREVIEW_CHARS: usize = 1500;
/// How many component sections the selected entity may render.
const MAX_DETAIL_ROWS: usize = 64;

/// Builds the ECS entity/component tree and the selected entity's editors.
///
/// Entity-first: the roots are live entities, and expanding one lists the
/// component types it holds. The selected entity additionally shows its
/// editable fields (or a read-only pretty value) inline, so the panel doubles
/// as the inspector.
pub fn ecs_panel<'a>(ecs: &World) -> Element<'a, DebugMessage, Theme, IcedRenderer> {
    let Some(state) = ecs.get_resource::<DebugUiState>() else {
        return crate::panels::blank();
    };

    if !state.is_panel_visible(PanelId::Ecs) {
        return crate::panels::blank();
    }

    // Owned, so the widget tree does not borrow the resource guard and the
    // returned element can be `'static` as `IcedState::with_view` requires.
    let filter = state.filter.clone();
    let mut rows = vec![
        text_input("filter (entity index, label or component name)", filter)
            .on_input(DebugMessage::SetFilter)
            .width(Length::Fill)
            .size(13)
            .into(),
    ];

    let needle = state.filter.trim().to_lowercase();
    let total = ecs.entity_count();
    let mut budget = MAX_ROWS;
    let mut detail_budget = MAX_DETAIL_ROWS;
    let mut matched = 0usize;

    for entity in ecs.entities() {
        let components = ecs.entity_components(&entity);
        let label = crate::panels::entity_label(ecs, &entity);
        let selected = state.selected == Some(entity);

        if !needle.is_empty() && !entity_matches(&entity, &components, label.as_deref(), &needle) {
            continue;
        }

        matched += 1;

        // The selected entity always gets its details, even when the name
        // budget is exhausted.
        if budget == 0 && !selected {
            continue;
        }
        budget = budget.saturating_sub(1);
        rows.push(entity_row(
            &state,
            &entity,
            components.len(),
            label.as_deref(),
        ));

        if !state.is_expanded(entity) {
            continue;
        }

        for (type_id, name) in components {
            if selected {
                if detail_budget == 0 {
                    break;
                }
                detail_budget -= 1;
                rows.push(component_details(ecs, &state, entity, type_id, name));
            } else {
                if budget == 0 {
                    break;
                }
                budget -= 1;
                rows.push(component_name(name));
            }
        }
    }

    if total > 0 && matched == 0 {
        rows.push(row![text("no entities match the filter").size(12)].into());
    }

    if let Some(error) = state.last_error.as_deref() {
        rows.push(crate::panels::divider());
        rows.push(
            text(error.to_string())
                .size(11)
                .color(Color::from_rgb(1.0, 0.5, 0.45))
                .into(),
        );
    }

    rows.push(
        text(format!(
            "{matched} of {total} entities · {MAX_ROWS} row budget"
        ))
        .size(11)
        .color(Color::from_rgb(0.6, 0.6, 0.6))
        .into(),
    );

    container(
        column![crate::panels::scrolling(crate::panels::column_of(
            rows, 1.0
        ))]
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

/// Whether an entity matches the filter, by index, label or component type name.
fn entity_matches(
    entity: &Entity,
    components: &[(TypeId, &'static str)],
    label: Option<&str>,
    needle: &str,
) -> bool {
    if entity.index.to_string().contains(needle) {
        return true;
    }

    if label.is_some_and(|label| label.to_lowercase().contains(needle)) {
        return true;
    }

    components
        .iter()
        .any(|(_, name)| name.to_lowercase().contains(needle))
}

fn entity_row(
    state: &DebugUiState,
    entity: &Entity,
    component_count: usize,
    label: Option<&str>,
) -> Element<'static, DebugMessage, Theme, IcedRenderer> {
    let expanded = state.is_expanded(*entity);
    let selected = state.selected == Some(*entity);

    let toggle = button(text(if expanded { "▾" } else { "▸" }).size(12))
        .on_press(DebugMessage::ToggleEntity(*entity))
        .width(Length::Shrink);

    // The selected entity is marked in its label: `Container` has no
    // background colour, and restyling the row would mean reaching into iced's
    // style closures.
    let mut label = match label {
        Some(name) => format!("{name}  ·  Entity {{ {} }}", entity.index),
        None => format!("Entity {{ {} }}", entity.index),
    };
    if selected {
        label.insert_str(0, "● ");
    }
    if component_count > 0 {
        label.push_str(&format!("  · {component_count} components"));
    }

    // Clicking the selected row again clears the selection and collapses it.
    let target = if selected { None } else { Some(*entity) };
    let select = button(text(label).size(13))
        .on_press(DebugMessage::SelectEntity(target))
        .width(Length::Fill);

    container(row![toggle, select].spacing(4).width(Length::Fill))
        .width(Length::Fill)
        .into()
}

fn component_name(name: &'static str) -> Element<'static, DebugMessage, Theme, IcedRenderer> {
    row![
        text("•").size(10).color(Color::from_rgb(0.5, 0.55, 0.65)),
        text(short_type_name(name))
            .size(11)
            .color(Color::from_rgb(0.7, 0.85, 1.0)),
    ]
    .spacing(4)
    .padding([0.0, 12.0])
    .into()
}

fn component_details(
    ecs: &World,
    state: &DebugUiState,
    entity: Entity,
    type_id: TypeId,
    name: &'static str,
) -> Element<'static, DebugMessage, Theme, IcedRenderer> {
    let header = text(short_type_name(name))
        .size(12)
        .color(Color::from_rgb(0.7, 0.85, 1.0));

    let body = if let Some(ops) = editable_fields_ops(type_id) {
        if type_id == TypeId::of::<ModelInstances>() {
            model_instances_editor(ecs, state, entity)
        } else {
            let fields = ecs
                .component_debug(type_id, &entity)
                .map(|handle| (ops.fields)(handle.as_any()))
                .unwrap_or_default();

            if fields.is_empty() {
                pretty_value(ecs, entity, type_id)
            } else {
                let rows = fields
                    .iter()
                    .map(|field| field_row(state, entity, type_id, field))
                    .collect::<Vec<_>>();

                column(rows).spacing(2).into()
            }
        }
    } else if is_editable(type_id) {
        match ecs
            .component_debug(type_id, &entity)
            .and_then(|handle| display_for(type_id, handle.as_any()))
        {
            Some(value) => whole_component_row(state, entity, type_id, value),
            None => pretty_value(ecs, entity, type_id),
        }
    } else {
        pretty_value(ecs, entity, type_id)
    };

    column![header, body]
        .spacing(1)
        .padding([2.0, 12.0])
        .width(Length::Fill)
        .into()
}

fn field_row(
    state: &DebugUiState,
    entity: Entity,
    type_id: TypeId,
    field: &EditableField,
) -> Element<'static, DebugMessage, Theme, IcedRenderer> {
    let key = FieldKey::new(entity, type_id, field.id.clone());
    field_input_row(state, key, &field.label, &field.value)
}

fn field_input_row(
    state: &DebugUiState,
    key: FieldKey,
    label: &str,
    current: &str,
) -> Element<'static, DebugMessage, Theme, IcedRenderer> {
    let buffer = state.field_buffer(&key, current);
    let input_key = key.clone();
    let submit_key = key.clone();

    let input = text_input(label.to_string(), buffer)
        .on_input(move |text| DebugMessage::EditField(input_key.clone(), text))
        .on_submit(DebugMessage::ApplyField(submit_key))
        .width(Length::Fill)
        .size(12);

    let apply = button(text("set").size(11))
        .on_press(DebugMessage::ApplyField(key))
        .width(Length::Shrink);

    column![
        text(label.to_string())
            .size(10)
            .color(Color::from_rgb(0.55, 0.55, 0.6)),
        row![input, apply]
            .spacing(4)
            .align_y(iced_winit::core::Alignment::Center),
    ]
    .spacing(0)
    .into()
}

fn whole_component_row(
    state: &DebugUiState,
    entity: Entity,
    type_id: TypeId,
    value: String,
) -> Element<'static, DebugMessage, Theme, IcedRenderer> {
    let key = FieldKey::new(entity, type_id, String::new());
    let label = editable_label(type_id).unwrap_or("value");
    field_input_row(state, key, label, &value)
}

fn pretty_value(
    ecs: &World,
    entity: Entity,
    type_id: TypeId,
) -> Element<'static, DebugMessage, Theme, IcedRenderer> {
    let value = ecs
        .component_debug_pretty_preview(type_id, &entity, DETAIL_PREVIEW_CHARS)
        .unwrap_or_else(|| "<unavailable>".to_string());

    container(
        text(value)
            .size(11)
            .font(Font::MONOSPACE)
            .width(Length::Fill)
            .wrapping(Wrapping::WordOrGlyph),
    )
    .padding(4)
    .width(Length::Fill)
    .style(iced_widget::container::bordered_box)
    .into()
}

fn model_instances_editor(
    ecs: &World,
    state: &DebugUiState,
    entity: Entity,
) -> Element<'static, DebugMessage, Theme, IcedRenderer> {
    let type_id = TypeId::of::<ModelInstances>();

    let instances = ecs
        .component_debug(type_id, &entity)
        .and_then(|handle| {
            handle
                .as_any()
                .downcast_ref::<ModelInstances>()
                .map(|instances| {
                    let mut entries = instances.0.iter().collect::<Vec<_>>();
                    entries.sort_by_key(|(ulid, _)| **ulid);
                    entries
                        .into_iter()
                        .map(|(ulid, transform)| (*ulid, *transform))
                        .collect::<Vec<_>>()
                })
        })
        .unwrap_or_default();

    let mut rows = vec![
        row![
            text(format!("{} instances", instances.len()))
                .size(10)
                .color(Color::from_rgb(0.55, 0.55, 0.6)),
            text(""),
            button(text("+ add").size(10)).on_press(DebugMessage::AddModelInstance(entity)),
        ]
        .spacing(6)
        .align_y(iced_winit::core::Alignment::Center)
        .into(),
    ];

    if instances.is_empty() {
        rows.push(
            text("no instances")
                .size(11)
                .color(Color::from_rgb(0.6, 0.6, 0.6))
                .into(),
        );
    }

    for (ulid, transform) in instances.iter().take(MODEL_INSTANCE_EDIT_LIMIT) {
        rows.push(
            row![
                text(short_model_instance_id(ulid))
                    .size(10)
                    .font(Font::MONOSPACE)
                    .color(Color::from_rgb(0.75, 0.8, 0.9)),
                text(""),
                button(text("×").size(11)).on_press(DebugMessage::RemoveModelInstance {
                    entity,
                    ulid: ulid.0,
                }),
            ]
            .spacing(4)
            .align_y(iced_winit::core::Alignment::Center)
            .into(),
        );

        let fields = [
            (
                "position",
                EditableValue::to_edit_string(&transform.position),
            ),
            ("rotation (deg)", euler_degrees_text(&transform.rotation)),
            ("scale", EditableValue::to_edit_string(&transform.scale)),
        ];

        for (field, value) in fields {
            let key = model_instance_field_id(ulid, field);
            let key = FieldKey::new(entity, type_id, key);
            rows.push(field_input_row(state, key, field, &value));
        }
    }

    if instances.len() > MODEL_INSTANCE_EDIT_LIMIT {
        rows.push(
            text(format!(
                "… {} more instances",
                instances.len() - MODEL_INSTANCE_EDIT_LIMIT
            ))
            .size(10)
            .color(Color::from_rgb(0.6, 0.6, 0.6))
            .into(),
        );
    }

    column(rows).spacing(2).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_matches_index_label_and_component_name() {
        let entity = Entity::new(12, 0);
        let components: Vec<(TypeId, &'static str)> = Vec::new();

        assert!(entity_matches(&entity, &components, None, "12"));
        assert!(!entity_matches(&entity, &components, None, "13"));
        assert!(entity_matches(
            &entity,
            &components,
            Some("Damaged Helmet"),
            "helmet"
        ));

        let components = vec![(TypeId::of::<String>(), "orbital_ecs_bridge::Position")];
        assert!(entity_matches(&entity, &components, None, "position"));
    }
}

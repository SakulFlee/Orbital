//! Panel listing live entities and their components.

use std::any::TypeId;

use iced_widget::{button, column, container, row, text, text_input};
use iced_winit::core::{Color, Element, Length, Theme};

use orbital_ecs::{Entity, World};

use crate::state::Renderer as IcedRenderer;
use crate::state::{DebugMessage, DebugUiState, MAX_ROWS, PanelId};

const PANEL_WIDTH: f32 = 380.0;

/// Builds the ECS entity/component tree.
///
/// Entity-first: the roots are live entities, and expanding one lists the
/// component types it holds with their current values. The world is flat —
/// nothing in the engine uses `Parent`/`Children` — so there is no scene-graph
/// shape to show.
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
        text_input("filter (entity index or component name)", filter)
            .on_input(DebugMessage::SetFilter)
            .width(Length::Fill)
            .size(13)
            .into(),
    ];

    let needle = state.filter.trim().to_lowercase();
    let total = ecs.entity_count();
    let mut budget = MAX_ROWS;
    let mut matched = 0usize;

    for entity in ecs.entities() {
        let components = ecs.entity_components(&entity);

        if !needle.is_empty() && !entity_matches(&entity, &components, &needle) {
            continue;
        }

        matched += 1;

        if budget == 0 {
            continue;
        }
        budget -= 1;
        rows.push(entity_row(&state, &entity, components.len()));

        if !state.is_expanded(entity) {
            continue;
        }

        for (type_id, name) in components {
            if budget == 0 {
                break;
            }
            budget -= 1;
            rows.push(component_row(ecs, entity, type_id, name));
        }
    }

    if total > 0 && matched == 0 {
        rows.push(row![text("no entities match the filter").size(12)].into());
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
    .width(PANEL_WIDTH)
    .into()
}

/// Whether an entity matches the filter, by index or by component type name.
fn entity_matches(entity: &Entity, components: &[(TypeId, &'static str)], needle: &str) -> bool {
    if entity.index.to_string().contains(needle) {
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
) -> Element<'static, DebugMessage, Theme, IcedRenderer> {
    let expanded = state.is_expanded(*entity);
    let selected = state.selected == Some(*entity);

    let toggle = button(text(if expanded { "▾" } else { "▸" }).size(12))
        .on_press(DebugMessage::ToggleEntity(*entity))
        .width(Length::Shrink);

    // The selected entity is marked in its label: `Container` has no
    // background colour, and restyling the row would mean reaching into iced's
    // style closures.
    let mut label = if selected {
        format!("● Entity {{ {} }}", entity.index)
    } else {
        format!("Entity {{ {} }}", entity.index)
    };
    if component_count > 0 {
        label.push_str(&format!("  · {component_count} components"));
    }

    let select = button(text(label).size(13))
        .on_press(DebugMessage::SelectEntity(Some(*entity)))
        .width(Length::Fill);

    container(row![toggle, select].spacing(4).width(Length::Fill))
        .width(Length::Fill)
        .into()
}

fn component_row(
    ecs: &World,
    entity: Entity,
    type_id: TypeId,
    name: &'static str,
) -> Element<'static, DebugMessage, Theme, IcedRenderer> {
    let value = ecs
        .component_debug_string(type_id, &entity)
        .map(|value| DebugUiState::preview(&value))
        .unwrap_or_else(|| "<unavailable>".to_string());

    column![
        text(short_type_name(name))
            .size(12)
            .color(Color::from_rgb(0.7, 0.85, 1.0)),
        text(value).size(12),
    ]
    .spacing(0)
    .padding(2)
    .width(Length::Fill)
    .into()
}

/// The last path segment of a type name, e.g. `Position` for
/// `my_game::components::Position`.
fn short_type_name(name: &str) -> String {
    name.rsplit("::").next().unwrap_or(name).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_type_name_strips_the_module_path() {
        assert_eq!(short_type_name("my_game::components::Position"), "Position");
        assert_eq!(short_type_name("Position"), "Position");
        // A trailing separator leaves an empty last segment.
        assert_eq!(short_type_name("a::b::"), "");
    }

    #[test]
    fn filter_matches_index_and_component_name() {
        let entity = Entity::new(12, 0);
        let components: Vec<(TypeId, &'static str)> = Vec::new();

        assert!(entity_matches(&entity, &components, "12"));
        assert!(!entity_matches(&entity, &components, "13"));
    }
}

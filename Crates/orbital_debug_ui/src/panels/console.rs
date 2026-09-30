//! Panel showing the tail of the in-memory log buffer.

use iced_widget::{button, column, container, row, text};
use iced_winit::core::{Color, Element, Length, Theme};
use log::LevelFilter;

use orbital_ecs::World;

use crate::state::Renderer as IcedRenderer;
use crate::state::{DebugMessage, DebugUiState, MAX_ROWS, PanelId};

const PANEL_WIDTH: f32 = 520.0;

const LEVELS: [LevelFilter; 5] = [
    LevelFilter::Off,
    LevelFilter::Error,
    LevelFilter::Warn,
    LevelFilter::Info,
    LevelFilter::Debug,
];

/// Builds the console panel from the log ring maintained by
/// `orbital_core::logging`.
///
/// The ring is read through `with_log_buffer`, so this does not copy the whole
/// buffer every frame.
pub fn console_panel<'a>(ecs: &World) -> Element<'a, DebugMessage, Theme, IcedRenderer> {
    let Some(state) = ecs.get_resource::<DebugUiState>() else {
        return crate::panels::blank();
    };

    if !state.is_panel_visible(PanelId::Console) {
        return crate::panels::blank();
    }

    let mut level_buttons = row![].spacing(4);
    for level in LEVELS {
        let selected = level == state.min_level;
        let label = format!("{level}");

        let button = if selected {
            button(text(label).size(11))
                .on_press(DebugMessage::SetMinLevel(level))
                .style(iced_widget::button::primary)
        } else {
            button(text(label).size(11)).on_press(DebugMessage::SetMinLevel(level))
        };

        level_buttons = level_buttons.push(button);
    }

    let header = row![
        text("level:").size(12),
        level_buttons,
        text(""),
        button(text("clear").size(11)).on_press(DebugMessage::ClearLog),
    ]
    .spacing(6)
    .align_y(iced_winit::core::Alignment::Center);

    // Collect newest-first, then reverse for display, so the most recent line is
    // at the bottom of the panel and the row budget is spent on recent output.
    let mut lines = orbital_core::logging::with_log_buffer(|buffer| {
        buffer
            .rev()
            .filter(|line| line.level <= state.min_level)
            .take(MAX_ROWS)
            .map(|line| {
                (
                    format!("[{}] {}", line.level, line.target),
                    line.message.clone(),
                    level_color(line.level),
                )
            })
            .collect::<Vec<_>>()
    });

    lines.reverse();

    if lines.is_empty() {
        lines.push((
            String::new(),
            format!("no log records at or below {}", state.min_level),
            Color::from_rgb(0.6, 0.6, 0.6),
        ));
    }

    let rows = lines
        .into_iter()
        .map(|(prefix, message, color)| {
            column![
                text(prefix)
                    .size(10)
                    .color(Color::from_rgb(0.55, 0.55, 0.6)),
                text(message).size(12).color(color),
            ]
            .spacing(0)
            .width(Length::Fill)
            .into()
        })
        .collect::<Vec<_>>();

    container(
        column![
            header,
            crate::panels::scrolling(crate::panels::column_of(rows, 2.0))
        ]
        .spacing(6)
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .width(PANEL_WIDTH)
    .into()
}

fn level_color(level: log::Level) -> Color {
    match level {
        log::Level::Error => Color::from_rgb(1.0, 0.45, 0.45),
        log::Level::Warn => Color::from_rgb(1.0, 0.8, 0.4),
        log::Level::Info => Color::from_rgb(0.85, 0.9, 0.95),
        log::Level::Debug => Color::from_rgb(0.7, 0.75, 0.8),
        log::Level::Trace => Color::from_rgb(0.55, 0.6, 0.65),
    }
}

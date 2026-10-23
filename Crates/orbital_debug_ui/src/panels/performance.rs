//! Panel showing frame rate and per-stage render timings.

use iced_widget::{button, canvas, column, row, text};
use iced_winit::core::{Alignment, Color, Element, Font, Length, Theme};

use orbital_ecs::World;
use orbital_ecs_bridge::{FpsStats, FrameTimings, TIMING_STAGES, TOTAL_STAGE_INDEX};

use crate::sparkline::{Line, Sparkline};
use crate::state::Renderer as IcedRenderer;
use crate::state::{DebugMessage, DebugUiState, GraphMetric, PanelId};

/// How many frames the rolling `avg`/`min`/`max` graphs look back.
const GRAPH_WINDOW: usize = 30;
const GRAPH_HEIGHT: f32 = 72.0;
const STAGE_COLUMN_WIDTH: f32 = 130.0;
const VALUE_COLUMN_WIDTH: f32 = 58.0;
const LAST_COLOR: Color = Color::from_rgb(0.5, 0.9, 0.6);
const GRAPH_COLOR: Color = LAST_COLOR;
const AVG_COLOR: Color = Color::from_rgb(0.4, 0.8, 1.0);
const MIN_COLOR: Color = Color::from_rgb(0.45, 0.6, 1.0);
const MAX_COLOR: Color = Color::from_rgb(1.0, 0.8, 0.4);

/// Builds the performance panel.
///
/// Reads `FpsStats` (updated once a second) and `FrameTimings` (updated every
/// frame), so this is worth rebuilding on a slow interval rather than per frame.
pub fn performance_panel<'a>(ecs: &World) -> Element<'a, DebugMessage, Theme, IcedRenderer> {
    let Some(state) = ecs.get_resource::<DebugUiState>() else {
        return crate::panels::blank();
    };

    if !state.is_panel_visible(PanelId::Performance) {
        return crate::panels::blank();
    }

    let mut sections = vec![crate::panels::caption(
        "FPS / frame time",
        Color::from_rgb(0.7, 0.8, 1.0),
    )];

    match ecs.get_resource::<FpsStats>() {
        Some(stats) => {
            sections.push(row![text(format!("FPS        {}", stats.fps)).size(13)].into());
            sections.push(
                row![text(format!("TDT        {:.2}s", stats.total_delta_time)).size(13)].into(),
            );
            sections.push(
                row![text(format!("CDT        {:.4}s", stats.cycle_delta_time)).size(13)].into(),
            );
        }
        None => sections.push(row![text("FPS        --").size(13)].into()),
    }

    sections.push(crate::panels::divider());

    match ecs.get_resource::<FrameTimings>() {
        Some(timings) => {
            sections.push(metric_selector(&state));

            if state.graph_stages.is_empty() {
                sections.push(
                    row![
                        text("no graphs pinned — click a stage below")
                            .size(11)
                            .color(Color::from_rgb(0.6, 0.6, 0.6))
                    ]
                    .into(),
                );
            } else {
                for &stage in &state.graph_stages {
                    if stage >= TIMING_STAGES.len() {
                        continue;
                    }

                    sections.push(graph(&timings, stage, state.graph_metric));
                }
            }

            sections.push(crate::panels::divider());
            sections.push(timing_header());

            for (index, sample) in timings.samples().iter().enumerate() {
                let pinned = state.graph_stages.contains(&index);
                let content = row![
                    text(sample.name)
                        .size(12)
                        .font(Font::MONOSPACE)
                        .width(Length::Fixed(STAGE_COLUMN_WIDTH)),
                    value_cell(format_ms(sample.last_ms)),
                    value_cell(format_ms(sample.avg_ms)),
                    value_cell(format_ms(sample.min_ms)),
                    value_cell(format_ms(sample.max_ms)),
                ]
                .spacing(4)
                .align_y(Alignment::Center);

                sections.push(
                    button(content)
                        .on_press(DebugMessage::ToggleGraphStage(index))
                        .padding(0)
                        .width(Length::Fill)
                        .style(move |theme, status| {
                            if pinned {
                                button::primary(theme, status)
                            } else {
                                button::text(theme, status)
                            }
                        })
                        .into(),
                );
            }

            sections.push(
                row![
                    text("click a stage to toggle its graph")
                        .size(10)
                        .color(Color::from_rgb(0.55, 0.55, 0.6))
                ]
                .into(),
            );

            sections.push(crate::panels::divider());
            sections.push(
                row![text(format!("{} frames recorded", timings.frame_count())).size(11)].into(),
            );

            // GPU timings only exist where the backend supports timestamp
            // queries; say so rather than showing a column of zeros.
            if timings.frame_count() > 0 && timings.samples()[TOTAL_STAGE_INDEX].max_ms == 0.0 {
                sections
                    .push(row![text("gpu timings unavailable on this backend").size(11)].into());
            }
        }
        None => sections.push(row![text("no timings recorded yet").size(12)].into()),
    }

    crate::panels::scrolling(
        crate::panels::column_of(sections, 2.0)
            .padding(10)
            .width(Length::Fill),
    )
}

fn metric_selector<'a>(state: &DebugUiState) -> Element<'a, DebugMessage, Theme, IcedRenderer> {
    let mut buttons = row![].spacing(4);

    for metric in [
        GraphMetric::Last,
        GraphMetric::Avg,
        GraphMetric::Min,
        GraphMetric::Max,
        GraphMetric::All,
    ] {
        let label = metric_label(metric);
        let selected = state.graph_metric == metric;

        let metric_button = if selected {
            button(text(label).size(11))
                .on_press(DebugMessage::SetGraphMetric(metric))
                .style(button::primary)
        } else {
            button(text(label).size(11)).on_press(DebugMessage::SetGraphMetric(metric))
        };

        buttons = buttons.push(metric_button);
    }

    row![text("metric:").size(12), buttons]
        .spacing(6)
        .align_y(Alignment::Center)
        .into()
}

fn graph<'a>(
    timings: &FrameTimings,
    stage: usize,
    metric: GraphMetric,
) -> Element<'a, DebugMessage, Theme, IcedRenderer> {
    let raw = timings
        .history()
        .iter()
        .map(|frame| frame.get(stage).copied().unwrap_or(0.0))
        .collect::<Vec<_>>();

    let last = raw.iter().map(|value| *value as f32).collect::<Vec<_>>();

    let sparkline = match metric {
        GraphMetric::Last => Sparkline::auto_scaled(last, GRAPH_COLOR),
        GraphMetric::Avg => Sparkline::auto_scaled(rolling(&raw, GRAPH_WINDOW, mean), GRAPH_COLOR),
        GraphMetric::Min => {
            Sparkline::auto_scaled(rolling(&raw, GRAPH_WINDOW, minimum), GRAPH_COLOR)
        }
        GraphMetric::Max => {
            Sparkline::auto_scaled(rolling(&raw, GRAPH_WINDOW, maximum), GRAPH_COLOR)
        }
        GraphMetric::All => Sparkline::layered(
            last,
            vec![
                Line::new(rolling(&raw, GRAPH_WINDOW, mean), AVG_COLOR, 1.0),
                Line::new(rolling(&raw, GRAPH_WINDOW, minimum), MIN_COLOR, 0.9),
                Line::new(rolling(&raw, GRAPH_WINDOW, maximum), MAX_COLOR, 0.9),
            ],
            false,
            LAST_COLOR,
        ),
    };

    let stage_name = if stage == TOTAL_STAGE_INDEX {
        "frame time"
    } else {
        TIMING_STAGES[stage]
    };

    let caption = format!(
        "{stage_name} · {} (ms, last {} frames)",
        metric_label(metric),
        raw.len()
    );

    let mut content = column![
        crate::panels::caption(&caption, GRAPH_COLOR),
        canvas::Canvas::new(sparkline)
            .width(Length::Fill)
            .height(GRAPH_HEIGHT)
    ]
    .spacing(1);

    if metric == GraphMetric::All {
        content = content.push(
            row![
                legend_item("last", LAST_COLOR),
                legend_item("avg", AVG_COLOR),
                legend_item("min", MIN_COLOR),
                legend_item("max", MAX_COLOR),
            ]
            .spacing(10),
        );
    }

    content.into()
}

fn legend_item(
    label: &'static str,
    color: Color,
) -> Element<'static, DebugMessage, Theme, IcedRenderer> {
    text(label).size(10).color(color).into()
}

fn timing_header<'a>() -> Element<'a, DebugMessage, Theme, IcedRenderer> {
    row![
        header_cell("stage", STAGE_COLUMN_WIDTH, Alignment::Start),
        header_cell("last", VALUE_COLUMN_WIDTH, Alignment::End),
        header_cell("avg", VALUE_COLUMN_WIDTH, Alignment::End),
        header_cell("min", VALUE_COLUMN_WIDTH, Alignment::End),
        header_cell("max", VALUE_COLUMN_WIDTH, Alignment::End),
    ]
    .spacing(4)
    .into()
}

fn header_cell<'a>(
    label: &'a str,
    width: f32,
    alignment: Alignment,
) -> Element<'a, DebugMessage, Theme, IcedRenderer> {
    text(label)
        .size(11)
        .color(Color::from_rgb(0.6, 0.6, 0.6))
        .width(Length::Fixed(width))
        .align_x(alignment)
        .into()
}

fn value_cell(value: String) -> Element<'static, DebugMessage, Theme, IcedRenderer> {
    text(value)
        .size(12)
        .font(Font::MONOSPACE)
        .width(Length::Fixed(VALUE_COLUMN_WIDTH))
        .align_x(Alignment::End)
        .into()
}

fn metric_label(metric: GraphMetric) -> &'static str {
    match metric {
        GraphMetric::Last => "last",
        GraphMetric::Avg => "avg",
        GraphMetric::Min => "min",
        GraphMetric::Max => "max",
        GraphMetric::All => "all",
    }
}

/// A rolling statistic over `values`, one output sample per input sample.
fn rolling(values: &[f64], window: usize, statistic: fn(&[f64]) -> f64) -> Vec<f32> {
    let window = window.max(1);

    values
        .iter()
        .enumerate()
        .map(|(index, _)| {
            let start = (index + 1).saturating_sub(window);
            statistic(&values[start..=index]) as f32
        })
        .collect()
}

fn mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

fn minimum(values: &[f64]) -> f64 {
    values.iter().copied().fold(f64::INFINITY, f64::min)
}

fn maximum(values: &[f64]) -> f64 {
    values.iter().copied().fold(f64::NEG_INFINITY, f64::max)
}

/// Formats a duration in milliseconds, using more precision as it gets small.
fn format_ms(value: f64) -> String {
    if value >= 10.0 {
        format!("{value:.2}")
    } else {
        format!("{value:.3}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_durations_keep_precision() {
        assert_eq!(format_ms(0.125), "0.125");
        assert_eq!(format_ms(9.999), "9.999");
    }

    #[test]
    fn large_durations_are_shortened() {
        assert_eq!(format_ms(10.0), "10.00");
        assert_eq!(format_ms(1234.5), "1234.50");
    }

    #[test]
    fn rolling_mean_uses_the_window() {
        assert_eq!(
            rolling(&[0.0, 10.0, 20.0, 30.0], 2, mean),
            vec![0.0, 5.0, 15.0, 25.0]
        );
    }

    #[test]
    fn rolling_min_and_max_use_the_window() {
        let values = [0.0, 10.0, 20.0, 30.0];

        assert_eq!(rolling(&values, 2, minimum), vec![0.0, 0.0, 10.0, 20.0]);
        assert_eq!(rolling(&values, 2, maximum), vec![0.0, 10.0, 20.0, 30.0]);
    }

    #[test]
    fn rolling_handles_a_window_larger_than_the_series() {
        assert_eq!(rolling(&[4.0, 8.0], 30, mean), vec![4.0, 6.0]);
    }
}

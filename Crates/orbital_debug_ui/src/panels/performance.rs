//! Panel showing frame rate and per-stage render timings.

use std::collections::VecDeque;

use iced_widget::{button, canvas, column, row, slider, text, text_input};
use iced_winit::core::{Alignment, Color, Element, Font, Length, Theme};

use orbital_ecs::World;
use orbital_ecs_bridge::{FpsStats, FrameTimings, TIMING_STAGES, TOTAL_STAGE_INDEX};

use crate::sparkline::{Line, Sparkline};
use crate::state::Renderer as IcedRenderer;
use crate::state::{
    DebugMessage, DebugUiState, GraphMetric, GraphSeries, MAX_GRAPH_WINDOW_SECS,
    MIN_GRAPH_WINDOW_SECS, PanelId, format_seconds,
};

const GRAPH_HEIGHT: f32 = 72.0;
const STAGE_COLUMN_WIDTH: f32 = 130.0;
const VALUE_COLUMN_WIDTH: f32 = 58.0;
/// Maximum number of points drawn per series.
const MAX_GRAPH_POINTS: usize = 512;
/// Rolling window used by the avg/min/max graph modes.
const ROLLING_WINDOW_SECS: f64 = 1.0;
const LAST_COLOR: Color = Color::from_rgb(0.5, 0.9, 0.6);
const AVG_COLOR: Color = Color::from_rgb(0.4, 0.8, 1.0);
const MIN_COLOR: Color = Color::from_rgb(0.45, 0.6, 1.0);
const MAX_COLOR: Color = Color::from_rgb(1.0, 0.8, 0.4);

/// Builds the performance panel.
///
/// Reads `FpsStats`, `FrameTimings` and the frame history retained while this
/// panel is visible.
pub fn performance_panel<'a>(ecs: &World) -> Element<'a, DebugMessage, Theme, IcedRenderer> {
    let Some(state) = ecs.get_resource::<DebugUiState>() else {
        return crate::panels::blank();
    };

    if !state.is_panel_visible(PanelId::Performance) {
        return crate::panels::blank();
    }

    let window_secs = state
        .graph_window_secs
        .clamp(MIN_GRAPH_WINDOW_SECS, MAX_GRAPH_WINDOW_SECS);

    let mut sections = vec![crate::panels::caption(
        "FPS / frame time",
        Color::from_rgb(0.7, 0.8, 1.0),
    )];

    match ecs.get_resource::<FpsStats>() {
        Some(stats) => {
            sections.push(row![text(format!("FPS (1s)   {}", stats.fps)).size(13)].into());
            sections.push(
                row![text(format!("TDT        {:.3}s", stats.total_delta_time)).size(13)].into(),
            );
            sections.push(
                row![
                    text(format!(
                        "CDT        {:.2}ms",
                        stats.cycle_delta_time * 1000.0
                    ))
                    .size(13)
                ]
                .into(),
            );
        }
        None => sections.push(row![text("FPS (1s)   --").size(13)].into()),
    }

    sections.push(crate::panels::divider());
    sections.push(range_control(&state));
    sections.push(metric_selector(&state));

    match ecs.get_resource::<FrameTimings>() {
        Some(timings) => {
            let start = timings.window_start(window_secs as f64);
            let window_frames = timings.history().len().saturating_sub(start);

            if state.graph_series.is_empty() {
                sections.push(
                    row![
                        text("no graphs pinned — click a row below")
                            .size(11)
                            .color(Color::from_rgb(0.6, 0.6, 0.6))
                    ]
                    .into(),
                );
            } else {
                for &series in &state.graph_series {
                    if matches!(series, GraphSeries::Stage(index) if index >= TIMING_STAGES.len()) {
                        continue;
                    }

                    sections.push(graph(
                        &timings,
                        series,
                        state.graph_metric,
                        start,
                        window_secs,
                    ));
                }
            }

            sections.push(crate::panels::divider());
            sections.push(crate::panels::caption(
                &format!(
                    "window stats · last {:.2}s · {} frames",
                    window_secs, window_frames
                ),
                Color::from_rgb(0.6, 0.6, 0.6),
            ));
            sections.push(timing_header());

            let fps_values = fps_values(&timings, start);
            sections.push(table_row(
                &state,
                GraphSeries::Fps,
                "FPS (inst)",
                stats(&fps_values),
                format_fps,
            ));

            let cdt_values = cdt_values(&timings, start);
            sections.push(table_row(
                &state,
                GraphSeries::Cdt,
                "CDT (ms)",
                stats(&cdt_values),
                format_ms_f32,
            ));

            for (index, sample) in timings.samples().iter().enumerate() {
                let values = stage_values(&timings, start, index);
                sections.push(table_row(
                    &state,
                    GraphSeries::Stage(index),
                    sample.name,
                    stats(&values),
                    format_ms_f32,
                ));
            }

            sections.push(
                row![
                    text("click a row to toggle its graph")
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

fn range_control<'a>(state: &DebugUiState) -> Element<'a, DebugMessage, Theme, IcedRenderer> {
    let seconds = state
        .graph_window_secs
        .clamp(MIN_GRAPH_WINDOW_SECS, MAX_GRAPH_WINDOW_SECS);
    let norm = window_norm(seconds);

    let slider = slider(0.0..=1.0, norm, move |norm| {
        DebugMessage::SetGraphWindow(seconds_from_norm(norm))
    })
    .step(0.001)
    .width(Length::Fill);

    let value = text(format!("{}s", format_seconds(seconds)))
        .size(12)
        .font(Font::MONOSPACE)
        .width(Length::Fixed(52.0))
        .align_x(Alignment::End);

    let mut rows = vec![
        row![text("window").size(12), slider, value]
            .spacing(6)
            .align_y(Alignment::Center)
            .into(),
        row![
            text("exact")
                .size(10)
                .color(Color::from_rgb(0.55, 0.55, 0.6)),
            text_input("seconds", state.graph_window_input.clone())
                .on_input(DebugMessage::EditGraphWindow)
                .on_submit(DebugMessage::ApplyGraphWindow)
                .width(Length::Fixed(72.0))
                .size(12)
                .style(crate::panels::input_style),
            text("s").size(11),
            button(text("set").size(11)).on_press(DebugMessage::ApplyGraphWindow),
        ]
        .spacing(4)
        .align_y(Alignment::Center)
        .into(),
    ];

    if let Some(error) = &state.graph_window_error {
        rows.push(
            text(error.clone())
                .size(10)
                .color(Color::from_rgb(1.0, 0.5, 0.45))
                .into(),
        );
    }

    column(rows).spacing(2).into()
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

fn graph(
    timings: &FrameTimings,
    series: GraphSeries,
    metric: GraphMetric,
    start: usize,
    window_secs: f32,
) -> Element<'static, DebugMessage, Theme, IcedRenderer> {
    let (values, label, unit, color) = match series {
        GraphSeries::Stage(index) => {
            let name = TIMING_STAGES.get(index).copied().unwrap_or("stage");
            let label = if index == TOTAL_STAGE_INDEX {
                "frame time"
            } else {
                name
            };

            (stage_values(timings, start, index), label, "ms", LAST_COLOR)
        }
        GraphSeries::Fps => (fps_values(timings, start), "FPS (inst)", "fps", AVG_COLOR),
        GraphSeries::Cdt => (cdt_values(timings, start), "CDT", "ms", MAX_COLOR),
    };

    let times = timings
        .history_times()
        .iter()
        .skip(start)
        .copied()
        .collect::<Vec<_>>();
    let rolling_secs = (window_secs as f64).min(ROLLING_WINDOW_SECS);
    let (average, minimum, maximum) = rolling_stats(&times, &values, rolling_secs);

    let sparkline = match metric {
        GraphMetric::Last => Sparkline::auto_scaled(downsample(&values, Downsample::Mean), color),
        GraphMetric::Avg => Sparkline::auto_scaled(downsample(&average, Downsample::Mean), color),
        GraphMetric::Min => Sparkline::auto_scaled(downsample(&minimum, Downsample::Min), color),
        GraphMetric::Max => Sparkline::auto_scaled(downsample(&maximum, Downsample::Max), color),
        GraphMetric::All => Sparkline::layered(
            downsample(&values, Downsample::Mean),
            vec![
                Line::new(downsample(&average, Downsample::Mean), AVG_COLOR, 1.0),
                Line::new(downsample(&minimum, Downsample::Min), MIN_COLOR, 0.9),
                Line::new(downsample(&maximum, Downsample::Max), MAX_COLOR, 0.9),
            ],
            false,
            LAST_COLOR,
        ),
    };

    let caption = format!(
        "{label} ({unit}, last {:.2}s, {} frames)",
        window_secs,
        values.len()
    );

    let mut content = column![
        crate::panels::caption(&caption, color),
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
        header_cell("series", STAGE_COLUMN_WIDTH, Alignment::Start),
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

fn table_row(
    state: &DebugUiState,
    series: GraphSeries,
    label: &'static str,
    stats: Option<Stats>,
    format: fn(f32) -> String,
) -> Element<'static, DebugMessage, Theme, IcedRenderer> {
    let value = |value: Option<f32>| value.map(format).unwrap_or_else(|| "--".to_string());

    let content = row![
        text(label)
            .size(12)
            .font(Font::MONOSPACE)
            .width(Length::Fixed(STAGE_COLUMN_WIDTH)),
        value_cell(value(stats.map(|stats| stats.last))),
        value_cell(value(stats.map(|stats| stats.avg))),
        value_cell(value(stats.map(|stats| stats.min))),
        value_cell(value(stats.map(|stats| stats.max))),
    ]
    .spacing(4)
    .align_y(Alignment::Center);

    let pinned = state.graph_series.contains(&series);

    button(content)
        .on_press(DebugMessage::ToggleGraphSeries(series))
        .padding(0)
        .width(Length::Fill)
        .style(move |theme, status| {
            if pinned {
                button::primary(theme, status)
            } else {
                button::text(theme, status)
            }
        })
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

fn stage_values(timings: &FrameTimings, start: usize, index: usize) -> Vec<f32> {
    timings
        .history()
        .iter()
        .skip(start)
        .map(|frame| frame.get(index).copied().unwrap_or(0.0) as f32)
        .collect()
}

fn fps_values(timings: &FrameTimings, start: usize) -> Vec<f32> {
    timings
        .history()
        .iter()
        .skip(start)
        .map(|frame| {
            let total = frame.get(TOTAL_STAGE_INDEX).copied().unwrap_or(0.0);

            if total > 0.0 {
                (1000.0 / total) as f32
            } else {
                0.0
            }
        })
        .collect()
}

fn cdt_values(timings: &FrameTimings, start: usize) -> Vec<f32> {
    timings.cdt_history().iter().skip(start).copied().collect()
}

#[derive(Clone, Copy)]
struct Stats {
    last: f32,
    avg: f32,
    min: f32,
    max: f32,
}

fn stats(values: &[f32]) -> Option<Stats> {
    let (&first, &last) = (values.first()?, values.last()?);
    let mut min = first;
    let mut max = first;
    let mut sum = 0.0f64;

    for &value in values {
        min = min.min(value);
        max = max.max(value);
        sum += value as f64;
    }

    Some(Stats {
        last,
        avg: (sum / values.len() as f64) as f32,
        min,
        max,
    })
}

#[derive(Clone, Copy)]
enum Downsample {
    Mean,
    Min,
    Max,
}

fn downsample(values: &[f32], mode: Downsample) -> Vec<f32> {
    if values.len() <= MAX_GRAPH_POINTS {
        return values.to_vec();
    }

    let buckets = MAX_GRAPH_POINTS;

    (0..buckets)
        .map(|bucket| {
            let start = bucket * values.len() / buckets;
            let end = ((bucket + 1) * values.len() / buckets).max(start + 1);
            let slice = &values[start..end.min(values.len())];

            match mode {
                Downsample::Mean => {
                    (slice.iter().map(|value| *value as f64).sum::<f64>() / slice.len() as f64)
                        as f32
                }
                Downsample::Min => slice.iter().copied().fold(f32::INFINITY, f32::min),
                Downsample::Max => slice.iter().copied().fold(f32::NEG_INFINITY, f32::max),
            }
        })
        .collect()
}

/// Rolling mean/min/max over a time window, one sample per input frame.
fn rolling_stats(
    times: &[f64],
    values: &[f32],
    window_secs: f64,
) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
    let len = times.len().min(values.len());
    let mut average = Vec::with_capacity(len);
    let mut minimum = Vec::with_capacity(len);
    let mut maximum = Vec::with_capacity(len);

    if len == 0 {
        return (average, minimum, maximum);
    }

    let window = window_secs.max(0.0);
    let mut start = 0usize;
    let mut sum = 0.0f64;
    let mut min_queue = VecDeque::new();
    let mut max_queue = VecDeque::new();

    for index in 0..len {
        sum += values[index] as f64;

        while start < index && times[index] - times[start] > window {
            sum -= values[start] as f64;
            start += 1;
        }

        while min_queue.front().is_some_and(|front| *front < start) {
            min_queue.pop_front();
        }
        while max_queue.front().is_some_and(|front| *front < start) {
            max_queue.pop_front();
        }

        while min_queue
            .back()
            .is_some_and(|back| values[*back] >= values[index])
        {
            min_queue.pop_back();
        }
        min_queue.push_back(index);

        while max_queue
            .back()
            .is_some_and(|back| values[*back] <= values[index])
        {
            max_queue.pop_back();
        }
        max_queue.push_back(index);

        let count = (index - start + 1) as f64;
        average.push((sum / count) as f32);
        minimum.push(values[*min_queue.front().expect("non-empty queue")]);
        maximum.push(values[*max_queue.front().expect("non-empty queue")]);
    }

    (average, minimum, maximum)
}

fn window_norm(seconds: f32) -> f32 {
    let seconds = seconds.clamp(MIN_GRAPH_WINDOW_SECS, MAX_GRAPH_WINDOW_SECS);

    ((seconds / MIN_GRAPH_WINDOW_SECS).ln() / (MAX_GRAPH_WINDOW_SECS / MIN_GRAPH_WINDOW_SECS).ln())
        .clamp(0.0, 1.0)
}

fn seconds_from_norm(norm: f32) -> f32 {
    let norm = norm.clamp(0.0, 1.0);
    let seconds =
        MIN_GRAPH_WINDOW_SECS * (MAX_GRAPH_WINDOW_SECS / MIN_GRAPH_WINDOW_SECS).powf(norm);

    (seconds.clamp(MIN_GRAPH_WINDOW_SECS, MAX_GRAPH_WINDOW_SECS) * 10.0).round() / 10.0
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

fn format_fps(value: f32) -> String {
    format!("{value:.1}")
}

fn format_ms_f32(value: f32) -> String {
    format_ms(value as f64)
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
    fn rolling_stats_use_the_time_window() {
        let times = [0.0, 1.0, 2.0];
        let values = [10.0, 20.0, 30.0];

        let (average, minimum, maximum) = rolling_stats(&times, &values, 1.0);

        assert_eq!(average, vec![10.0, 15.0, 25.0]);
        assert_eq!(minimum, vec![10.0, 10.0, 20.0]);
        assert_eq!(maximum, vec![10.0, 20.0, 30.0]);
    }

    #[test]
    fn rolling_stats_can_keep_only_the_current_frame() {
        let times = [0.0, 1.0, 2.0];
        let values = [10.0, 20.0, 30.0];

        let (average, minimum, maximum) = rolling_stats(&times, &values, 0.5);

        assert_eq!(average, vec![10.0, 20.0, 30.0]);
        assert_eq!(minimum, vec![10.0, 20.0, 30.0]);
        assert_eq!(maximum, vec![10.0, 20.0, 30.0]);
    }

    #[test]
    fn downsampling_compresses_but_preserves_modes() {
        let values = (0..2000).map(|value| value as f32).collect::<Vec<_>>();

        let mean = downsample(&values, Downsample::Mean);
        let min = downsample(&values, Downsample::Min);
        let max = downsample(&values, Downsample::Max);

        assert_eq!(mean.len(), MAX_GRAPH_POINTS);
        assert_eq!(min.len(), MAX_GRAPH_POINTS);
        assert_eq!(max.len(), MAX_GRAPH_POINTS);
        assert!(min.iter().all(|value| *value >= 0.0));
        assert!(max.iter().all(|value| *value <= 1999.0));
        assert!(mean.first().unwrap() < mean.last().unwrap());
    }

    #[test]
    fn the_window_slider_mapping_round_trips() {
        for seconds in [0.5, 1.0, 10.0, 30.0, 60.0] {
            let mapped = seconds_from_norm(window_norm(seconds));
            assert!(
                (mapped - seconds).abs() < 0.11,
                "mapped {seconds} to {mapped}"
            );
        }
    }

    #[test]
    fn stats_use_the_whole_slice() {
        let stats = stats(&[4.0, 2.0, 8.0]).expect("stats");

        assert_eq!(stats.last, 8.0);
        assert_eq!(stats.min, 2.0);
        assert_eq!(stats.max, 8.0);
        assert!((stats.avg - 14.0 / 3.0).abs() < 1e-5);
    }
}

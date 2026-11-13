//! Panel showing frame rate and per-stage render timings.

use std::collections::VecDeque;

use iced_widget::{button, canvas, checkbox, column, row, slider, text, text_input};
use iced_winit::core::{Alignment, Color, Element, Length, Theme};

use orbital_app::{PERFORMANCE_CSV_PATH, PERFORMANCE_METADATA_PATH};
use orbital_ecs::World;
use orbital_ecs_bridge::{
    FpsStats, FrameTimings, GPU_STAGE_START, PerformanceLogConfig, STAGE_FRAME_GAP,
    STAGE_FRAME_LIMITER, STAGE_GPU_COMPLETE, STAGE_GPU_SCENE_TOTAL, STAGE_PRESENT,
    STAGE_SURFACE_ACQ, TIMING_STAGES,
};

use crate::sparkline::{Line, Sparkline};
use crate::state::Renderer as IcedRenderer;
use crate::state::{
    DebugMessage, DebugUiState, GraphMetric, GraphSeries, MAX_GRAPH_WINDOW_SECS,
    MIN_GRAPH_WINDOW_SECS, format_seconds,
};

const GRAPH_HEIGHT: f32 = 100.0;
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

/// Healthy, marginal and slow value colours.
const GOOD_COLOR: Color = Color::from_rgb(0.5, 0.9, 0.6);
const WARN_COLOR: Color = Color::from_rgb(1.0, 0.8, 0.4);
const BAD_COLOR: Color = Color::from_rgb(1.0, 0.5, 0.45);

/// Instantaneous frame rate at or above this is healthy.
const FPS_GOOD: f32 = 60.0;
/// Frame rate at or above this is a warning.
const FPS_WARN: f32 = 30.0;

/// Whole-frame budget at 60 FPS, in milliseconds.
const FRAME_GOOD_MS: f32 = 1000.0 / 60.0;
/// Whole-frame budget at 30 FPS, in milliseconds.
const FRAME_WARN_MS: f32 = 1000.0 / 30.0;

/// CPU stage budget that is healthy, in milliseconds.
const CPU_STAGE_GOOD_MS: f32 = 1.0;
/// CPU stage budget that is still a warning, in milliseconds.
const CPU_STAGE_WARN_MS: f32 = 2.0;

/// Individual GPU pass budget that is healthy, in milliseconds.
const GPU_STAGE_GOOD_MS: f32 = 2.0;
/// Individual GPU pass budget that is still a warning, in milliseconds.
const GPU_STAGE_WARN_MS: f32 = 4.0;

/// How healthy a timing value is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Health {
    Good,
    Warn,
    Bad,
}

impl Health {
    fn color(self) -> Color {
        match self {
            Self::Good => GOOD_COLOR,
            Self::Warn => WARN_COLOR,
            Self::Bad => BAD_COLOR,
        }
    }
}

/// Good/warning limits for one kind of value.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Thresholds {
    good: f32,
    warn: f32,
    higher_is_better: bool,
}

impl Thresholds {
    const fn lower_is_better(good: f32, warn: f32) -> Self {
        Self {
            good,
            warn,
            higher_is_better: false,
        }
    }

    const fn higher_is_better(good: f32, warn: f32) -> Self {
        Self {
            good,
            warn,
            higher_is_better: true,
        }
    }

    /// Classifies `value`, or returns `None` if it is not a finite number.
    fn classify(self, value: f32) -> Option<Health> {
        if !value.is_finite() {
            return None;
        }

        if self.higher_is_better {
            if value >= self.good {
                Some(Health::Good)
            } else if value >= self.warn {
                Some(Health::Warn)
            } else {
                Some(Health::Bad)
            }
        } else if value <= self.good {
            Some(Health::Good)
        } else if value <= self.warn {
            Some(Health::Warn)
        } else {
            Some(Health::Bad)
        }
    }
}

const FPS_THRESHOLDS: Thresholds = Thresholds::higher_is_better(FPS_GOOD, FPS_WARN);
const FRAME_THRESHOLDS: Thresholds = Thresholds::lower_is_better(FRAME_GOOD_MS, FRAME_WARN_MS);
const CPU_STAGE_THRESHOLDS: Thresholds =
    Thresholds::lower_is_better(CPU_STAGE_GOOD_MS, CPU_STAGE_WARN_MS);
const GPU_STAGE_THRESHOLDS: Thresholds =
    Thresholds::lower_is_better(GPU_STAGE_GOOD_MS, GPU_STAGE_WARN_MS);

/// Picks the thresholds for one table row.
fn thresholds_for(series: GraphSeries) -> Thresholds {
    match series {
        GraphSeries::Fps => FPS_THRESHOLDS,
        GraphSeries::Cdt => FRAME_THRESHOLDS,
        GraphSeries::Stage(index) if index >= GPU_STAGE_START => {
            if index == STAGE_GPU_SCENE_TOTAL {
                FRAME_THRESHOLDS
            } else {
                GPU_STAGE_THRESHOLDS
            }
        }
        GraphSeries::Stage(index)
            if index == STAGE_PRESENT
                || index == STAGE_SURFACE_ACQ
                || index == STAGE_FRAME_GAP
                || index == STAGE_FRAME_LIMITER
                || index == STAGE_GPU_COMPLETE =>
        {
            FRAME_THRESHOLDS
        }
        GraphSeries::Stage(_) => CPU_STAGE_THRESHOLDS,
    }
}

/// The health of a row value, unless the row has no data to colour.
fn health_for(series: GraphSeries, value: f32, available: bool) -> Option<Health> {
    if !available {
        return None;
    }

    thresholds_for(series).classify(value)
}

/// Builds the performance panel.
///
/// Reads `FpsStats`, `FrameTimings` and the frame history retained while this
/// panel is visible.
pub fn performance_panel<'a>(ecs: &World) -> Element<'a, DebugMessage, Theme, IcedRenderer> {
    let Some(state) = ecs.get_resource::<DebugUiState>() else {
        return crate::panels::blank();
    };

    let window_secs = state
        .graph_window_secs
        .clamp(MIN_GRAPH_WINDOW_SECS, MAX_GRAPH_WINDOW_SECS);
    let log_config = ecs
        .get_resource::<PerformanceLogConfig>()
        .map(|config| *config)
        .unwrap_or_default();

    let mut sections = vec![crate::panels::caption(
        "FPS / frame time",
        Color::from_rgb(0.7, 0.8, 1.0),
    )];

    match ecs.get_resource::<FpsStats>() {
        Some(stats) => {
            sections.push(
                row![summary_text(
                    format!("FPS (1s)   {}", stats.fps),
                    stats.fps as f32,
                    FPS_THRESHOLDS,
                )]
                .into(),
            );
            sections.push(
                row![text(format!("TDT        {:.3}s", stats.total_delta_time)).size(13)].into(),
            );
            sections.push(
                row![summary_text(
                    format!("CDT        {:.2}ms", stats.cycle_delta_time * 1000.0),
                    (stats.cycle_delta_time * 1000.0) as f32,
                    FRAME_THRESHOLDS,
                )]
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
            let gpu_available =
                timings.frame_count() > 0 && timings.samples()[STAGE_GPU_SCENE_TOTAL].max_ms > 0.0;

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
            sections.extend(logging_section(log_config));
            sections.push(crate::panels::divider());
            sections.push(crate::panels::caption(
                &format!(
                    "window stats · last {:.2}s · {} frames",
                    window_secs, window_frames
                ),
                Color::from_rgb(0.6, 0.6, 0.6),
            ));
            sections.push(health_legend());
            sections.push(timing_header());

            let fps_values = fps_values(&timings, start);
            sections.push(table_row(
                &state,
                GraphSeries::Fps,
                "FPS (timer)",
                stats(&fps_values),
                format_fps,
                true,
            ));

            let cdt_values = cdt_values(&timings, start);
            sections.push(table_row(
                &state,
                GraphSeries::Cdt,
                "frame time (CDT)",
                stats(&cdt_values),
                format_ms_f32,
                true,
            ));

            for (index, sample) in timings.samples().iter().enumerate() {
                let values = stage_values(&timings, start, index);
                sections.push(table_row(
                    &state,
                    GraphSeries::Stage(index),
                    sample.name,
                    stats(&values),
                    format_ms_f32,
                    index < GPU_STAGE_START || gpu_available,
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
            // queries; say so rather than showing a column of zeros. The
            // health flag also distinguishes a missing backend from a sample
            // that has stopped advancing because the readback is behind.
            let gpu_health = timings.gpu_timestamp_health();
            if gpu_health.supported() {
                if gpu_health.is_stale() {
                    sections.push(
                        row![
                            text(format!(
                                "gpu timestamps stale for {} frames",
                                gpu_health.stale_frames()
                            ))
                            .size(11)
                            .color(Color::from_rgb(0.95, 0.65, 0.25))
                        ]
                        .into(),
                    );
                } else if gpu_available {
                    sections.push(
                        row![
                            text("gpu timestamps healthy")
                                .size(11)
                                .color(Color::from_rgb(0.45, 0.75, 0.45))
                        ]
                        .into(),
                    );
                } else if timings.frame_count() > 0 {
                    sections.push(
                        row![text("gpu timestamps waiting for the first sample").size(11)].into(),
                    );
                }
            } else if timings.frame_count() > 0 {
                sections
                    .push(row![text("gpu timings unavailable on this backend").size(11)].into());
            }
        }
        None => {
            sections.push(row![text("no timings recorded yet").size(12)].into());
            sections.push(crate::panels::divider());
            sections.extend(logging_section(log_config));
        }
    }

    crate::panels::scrolling(
        crate::panels::column_of(sections, 2.0)
            .padding(10)
            .width(Length::Fill),
    )
}

/// The logging toggles, kept below the graph so measurements stay in view.
fn logging_section<'a>(
    log_config: PerformanceLogConfig,
) -> Vec<Element<'a, DebugMessage, Theme, IcedRenderer>> {
    vec![
        crate::panels::caption("Logging", Color::from_rgb(0.6, 0.6, 0.6)),
        checkbox(log_config.console)
            .label("Log FPS, timings & cache counters")
            .text_size(12)
            .on_toggle(DebugMessage::SetPerformanceLogging)
            .into(),
        row![
            text("writes ~3 lines per second")
                .size(10)
                .color(Color::from_rgb(0.55, 0.55, 0.6))
        ]
        .into(),
        checkbox(log_config.csv)
            .label("Write performance CSV")
            .text_size(12)
            .on_toggle(DebugMessage::SetPerformanceCsv)
            .into(),
        row![
            text(format!("path: {PERFORMANCE_CSV_PATH}"))
                .size(10)
                .color(Color::from_rgb(0.55, 0.55, 0.6))
        ]
        .into(),
        row![
            text(format!("meta: {PERFORMANCE_METADATA_PATH}"))
                .size(10)
                .color(Color::from_rgb(0.55, 0.55, 0.6))
        ]
        .into(),
    ]
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

            (stage_values(timings, start, index), name, "ms", LAST_COLOR)
        }
        GraphSeries::Fps => (fps_values(timings, start), "FPS (timer)", "fps", AVG_COLOR),
        GraphSeries::Cdt => (cdt_values(timings, start), "frame time", "ms", MAX_COLOR),
    };

    let times = timings
        .history_times()
        .iter()
        .skip(start)
        .copied()
        .collect::<Vec<_>>();
    let rolling_secs = (window_secs as f64).min(ROLLING_WINDOW_SECS);
    let (average, minimum, maximum) = rolling_stats(&times, &values, rolling_secs);

    let thresholds = thresholds_for(series);
    let last = downsample(&values, Downsample::Mean);
    let avg = downsample(&average, Downsample::Mean);
    let min = downsample(&minimum, Downsample::Min);
    let max = downsample(&maximum, Downsample::Max);

    let peak = match metric {
        GraphMetric::Last => percentile_peak(&last, 0.95),
        GraphMetric::Avg => percentile_peak(&avg, 0.95),
        GraphMetric::Min => percentile_peak(&min, 0.95),
        GraphMetric::Max => percentile_peak(&max, 0.95),
        GraphMetric::All => [&last, &avg, &min, &max]
            .into_iter()
            .map(|values| percentile_peak(values, 0.95))
            .fold(0.0f32, f32::max),
    };

    // Bring a threshold into view when it is close to the data, but do not let
    // a distant target squash the curve against the floor.
    let scale_peak = [thresholds.good, thresholds.warn]
        .into_iter()
        .filter(|threshold| *threshold <= peak * 2.0)
        .fold(peak, f32::max);

    let sparkline = match metric {
        GraphMetric::Last => Sparkline::auto_scaled(last, color),
        GraphMetric::Avg => Sparkline::auto_scaled(avg, color),
        GraphMetric::Min => Sparkline::auto_scaled(min, color),
        GraphMetric::Max => Sparkline::auto_scaled(max, color),
        GraphMetric::All => Sparkline::layered(
            last,
            vec![
                Line::new(avg, AVG_COLOR, 1.0),
                Line::new(min, MIN_COLOR, 0.9),
                Line::new(max, MAX_COLOR, 0.9),
            ],
            false,
            LAST_COLOR,
        ),
    }
    .with_ceiling(scale_peak)
    .with_axis(unit, window_secs)
    .with_reference(
        thresholds.good,
        GOOD_COLOR,
        threshold_label(series, thresholds.good, unit),
    )
    .with_reference(
        thresholds.warn,
        WARN_COLOR,
        threshold_label(series, thresholds.warn, unit),
    );

    let caption = format!(
        "{label} · {unit} · last {:.2}s · {} frames",
        window_secs,
        values.len()
    );
    let summary = stats(&values)
        .map(|stats| {
            format!(
                "last {} · avg {} · min {} · max {}",
                format_graph_value(stats.last, unit),
                format_graph_value(stats.avg, unit),
                format_graph_value(stats.min, unit),
                format_graph_value(stats.max, unit),
            )
        })
        .unwrap_or_else(|| "no samples in window".to_string());

    let mut content = column![
        crate::panels::caption(&caption, color),
        text(summary)
            .size(10)
            .color(Color::from_rgb(0.62, 0.66, 0.72)),
        canvas::Canvas::new(sparkline)
            .width(Length::Fill)
            .height(GRAPH_HEIGHT)
    ]
    .spacing(2);

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

/// Colours a summary value by its health.
fn summary_text(
    content: String,
    value: f32,
    thresholds: Thresholds,
) -> Element<'static, DebugMessage, Theme, IcedRenderer> {
    let widget = text(content).size(13);

    match thresholds.classify(value) {
        Some(health) => widget.color(health.color()).into(),
        None => widget.into(),
    }
}

/// Legend explaining the health colours used in the table.
fn health_legend() -> Element<'static, DebugMessage, Theme, IcedRenderer> {
    row![
        text("health:")
            .size(10)
            .color(Color::from_rgb(0.6, 0.6, 0.6)),
        legend_item("good", GOOD_COLOR),
        legend_item("watch", WARN_COLOR),
        legend_item("slow", BAD_COLOR),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
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
    available: bool,
) -> Element<'static, DebugMessage, Theme, IcedRenderer> {
    let cell = |value: Option<f32>| {
        value
            .map(|value| value_cell(format(value), health_for(series, value, available)))
            .unwrap_or_else(|| value_cell("--".to_string(), None))
    };

    let content = row![
        text(label)
            .size(12)
            .width(Length::Fixed(STAGE_COLUMN_WIDTH)),
        cell(stats.map(|stats| stats.last)),
        cell(stats.map(|stats| stats.avg)),
        cell(stats.map(|stats| stats.min)),
        cell(stats.map(|stats| stats.max)),
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

fn value_cell(
    value: String,
    health: Option<Health>,
) -> Element<'static, DebugMessage, Theme, IcedRenderer> {
    let cell = text(value)
        .size(12)
        .width(Length::Fixed(VALUE_COLUMN_WIDTH))
        .align_x(Alignment::End);

    match health {
        Some(health) => cell.color(health.color()).into(),
        None => cell.into(),
    }
}

fn stage_values(timings: &FrameTimings, start: usize, index: usize) -> Vec<f32> {
    timings
        .history()
        .iter()
        .skip(start)
        .map(|frame| frame.get(index).copied().unwrap_or(0.0) as f32)
        .collect()
}

/// Instantaneous FPS from the timer's wall-clock frame delta.
///
/// This deliberately does not use the renderer-only `render_total` stage:
/// update, surface acquisition and event-loop time are part of the frame
/// period, so deriving FPS from render time over-reports it. Averaging these
/// samples over one second matches the CSV `fps` column.
fn fps_values(timings: &FrameTimings, start: usize) -> Vec<f32> {
    timings
        .cdt_history()
        .iter()
        .skip(start)
        .map(|&cdt_ms| if cdt_ms > 0.0 { 1000.0 / cdt_ms } else { 0.0 })
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

/// A high percentile of a series, so a handful of startup hitches does not
/// dictate the graph's whole Y scale.
fn percentile_peak(values: &[f32], fraction: f32) -> f32 {
    let mut sorted = values
        .iter()
        .copied()
        .filter(|value| value.is_finite())
        .collect::<Vec<_>>();

    if sorted.is_empty() {
        return 0.0;
    }

    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let index = ((sorted.len() - 1) as f32 * fraction.clamp(0.0, 1.0)).round() as usize;

    sorted[index.min(sorted.len() - 1)]
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

/// Formats a graph value without its unit, matching the graph's magnitude.
fn format_graph_value(value: f32, unit: &str) -> String {
    if unit == "fps" {
        format!("{value:.0}")
    } else {
        format_ms_f32(value)
    }
}

/// Whether a series' thresholds are whole-frame budgets.
fn is_frame_series(series: GraphSeries) -> bool {
    matches!(
        series,
        GraphSeries::Fps
            | GraphSeries::Cdt
            | GraphSeries::Stage(STAGE_PRESENT)
            | GraphSeries::Stage(STAGE_SURFACE_ACQ)
            | GraphSeries::Stage(STAGE_GPU_SCENE_TOTAL)
    )
}

/// Formats a threshold, using FPS for frame budgets and milliseconds for stages.
fn threshold_label(series: GraphSeries, value: f32, unit: &str) -> String {
    if is_frame_series(series) {
        let fps = if unit == "fps" {
            value
        } else {
            1000.0 / value.max(f32::EPSILON)
        };

        format!("{fps:.0} fps")
    } else {
        format!("{value:.1} ms")
    }
}

#[cfg(test)]
mod tests {
    use orbital_ecs_bridge::RENDER_TOTAL_STAGE_INDEX;

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
    fn graph_values_drop_noise() {
        assert_eq!(format_graph_value(342.4, "fps"), "342");
        assert_eq!(format_graph_value(3.021, "ms"), "3.021");
        assert_eq!(threshold_label(GraphSeries::Fps, 60.0, "fps"), "60 fps");
        assert_eq!(threshold_label(GraphSeries::Cdt, 16.666, "ms"), "60 fps");
        assert_eq!(
            threshold_label(GraphSeries::Stage(RENDER_TOTAL_STAGE_INDEX), 16.666, "ms"),
            "16.7 ms"
        );

        use orbital_ecs_bridge::STAGE_UPDATE;
        assert_eq!(
            threshold_label(GraphSeries::Stage(STAGE_UPDATE), 1.0, "ms"),
            "1.0 ms"
        );
    }

    #[test]
    fn fps_uses_the_wall_clock_frame_delta_not_render_total() {
        let mut timings = FrameTimings::new();
        let mut durations = vec![0.0; TIMING_STAGES.len()];
        durations[RENDER_TOTAL_STAGE_INDEX] = 4.0;

        timings.record_at(
            &durations,
            0.0,
            16.0,
            orbital_ecs_bridge::TimingsHistoryConfig {
                enabled: true,
                window_secs: 10.0,
            },
        );

        // 1000 / 16 ms = 62.5 FPS. Deriving from the 4 ms render total would
        // incorrectly report 250 FPS.
        assert_eq!(fps_values(&timings, 0), vec![62.5]);
        assert!(fps_values(&timings, 1).is_empty());

        timings.record_at(
            &durations,
            1.0,
            0.0,
            orbital_ecs_bridge::TimingsHistoryConfig {
                enabled: true,
                window_secs: 10.0,
            },
        );
        assert_eq!(
            fps_values(&timings, 1),
            vec![0.0],
            "zero deltas are skipped"
        );
    }

    #[test]
    fn percentile_peaks_ignore_non_finite_values_and_spikes() {
        assert_eq!(
            percentile_peak(&[1.0, f32::NAN, 3.0, f32::INFINITY, -2.0], 0.95),
            3.0
        );
        assert_eq!(percentile_peak(&[], 0.95), 0.0);

        let mut values = vec![1.0; 100];
        values[99] = 1000.0;
        assert_eq!(
            percentile_peak(&values, 0.95),
            1.0,
            "a single spike must not set the scale"
        );
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

    #[test]
    fn fps_thresholds_treat_higher_as_better() {
        assert_eq!(FPS_THRESHOLDS.classify(60.0), Some(Health::Good));
        assert_eq!(FPS_THRESHOLDS.classify(59.9), Some(Health::Warn));
        assert_eq!(FPS_THRESHOLDS.classify(30.0), Some(Health::Warn));
        assert_eq!(FPS_THRESHOLDS.classify(29.9), Some(Health::Bad));
    }

    #[test]
    fn timing_thresholds_treat_lower_as_better() {
        assert_eq!(CPU_STAGE_THRESHOLDS.classify(1.0), Some(Health::Good));
        assert_eq!(CPU_STAGE_THRESHOLDS.classify(1.5), Some(Health::Warn));
        assert_eq!(CPU_STAGE_THRESHOLDS.classify(2.0), Some(Health::Warn));
        assert_eq!(CPU_STAGE_THRESHOLDS.classify(2.1), Some(Health::Bad));

        assert_eq!(FRAME_THRESHOLDS.classify(FRAME_GOOD_MS), Some(Health::Good));
        assert_eq!(FRAME_THRESHOLDS.classify(FRAME_WARN_MS), Some(Health::Warn));
        assert_eq!(FRAME_THRESHOLDS.classify(34.0), Some(Health::Bad));
    }

    #[test]
    fn non_finite_values_are_not_coloured() {
        for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert_eq!(FPS_THRESHOLDS.classify(value), None);
            assert_eq!(FRAME_THRESHOLDS.classify(value), None);
        }
    }

    #[test]
    fn rows_pick_the_right_threshold_group() {
        use orbital_ecs_bridge::{STAGE_GPU_SHADOW, STAGE_UPDATE};

        assert_eq!(thresholds_for(GraphSeries::Fps), FPS_THRESHOLDS);
        assert_eq!(thresholds_for(GraphSeries::Cdt), FRAME_THRESHOLDS);
        assert_eq!(
            thresholds_for(GraphSeries::Stage(RENDER_TOTAL_STAGE_INDEX)),
            CPU_STAGE_THRESHOLDS,
            "render total is a stage, not the whole frame"
        );
        assert_eq!(
            thresholds_for(GraphSeries::Stage(STAGE_SURFACE_ACQ)),
            FRAME_THRESHOLDS
        );
        assert_eq!(
            thresholds_for(GraphSeries::Stage(STAGE_PRESENT)),
            FRAME_THRESHOLDS
        );
        assert_eq!(
            thresholds_for(GraphSeries::Stage(STAGE_GPU_SCENE_TOTAL)),
            FRAME_THRESHOLDS
        );
        assert_eq!(
            thresholds_for(GraphSeries::Stage(STAGE_GPU_SHADOW)),
            GPU_STAGE_THRESHOLDS
        );
        assert_eq!(
            thresholds_for(GraphSeries::Stage(STAGE_UPDATE)),
            CPU_STAGE_THRESHOLDS
        );
    }

    #[test]
    fn unavailable_rows_are_not_coloured() {
        use orbital_ecs_bridge::STAGE_GPU_SHADOW;

        assert_eq!(
            health_for(GraphSeries::Stage(STAGE_GPU_SHADOW), 0.0, false),
            None
        );
        assert_eq!(
            health_for(GraphSeries::Stage(STAGE_GPU_SHADOW), 0.0, true),
            Some(Health::Good)
        );
    }
}

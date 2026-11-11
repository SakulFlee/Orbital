//! A minimal line graph for the debug panels.
//!
//! `plotters-iced` would be the obvious choice, but its latest release (0.11.0)
//! requires `iced` 0.13 while this workspace runs a 0.15-dev fork, so it would
//! need forking too. The `canvas` widget is already present in the fork behind a
//! feature flag and needs no new dependencies, and a sparkline is a small
//! enough job that a hand-written `Program` is cheaper than the maintenance.

use iced_widget::canvas::{self, LineDash, Path, Program, Stroke};
use iced_widget::core::{Point, Rectangle, Size, alignment, text};
use iced_winit::core::{Color, Font, Pixels};

use crate::state::DebugMessage;
use crate::state::Renderer as IcedRenderer;

const AXIS_LEFT: f32 = 44.0;
const AXIS_RIGHT: f32 = 6.0;
const AXIS_TOP: f32 = 8.0;
const AXIS_BOTTOM: f32 = 18.0;
const AXIS_LABEL_SIZE: f32 = 10.0;
const REFERENCE_LABEL_SIZE: f32 = 9.0;
const GRID_COLOR: Color = Color::from_rgba(0.75, 0.8, 0.9, 0.16);
const MINOR_GRID_COLOR: Color = Color::from_rgba(0.75, 0.8, 0.9, 0.08);
const AXIS_COLOR: Color = Color::from_rgba(0.75, 0.8, 0.9, 0.35);
const AXIS_LABEL_COLOR: Color = Color::from_rgba(0.75, 0.78, 0.85, 0.85);
const DASH_SEGMENTS: &[f32] = &[3.0, 3.0];

/// The number of horizontal intervals the Y axis is divided into.
const AXIS_INTERVALS: usize = 4;

/// The horizontal ticks drawn across the plot.
const TIME_INTERVALS: usize = 4;

/// A Y-axis scale with human-readable steps.
#[derive(Debug, Clone, Copy)]
struct AxisScale {
    /// The value at the top of the plot. Values above it are clamped.
    max: f32,
    /// The distance between labelled grid lines.
    step: f32,
    /// Decimal places used for the labels.
    decimals: usize,
}

impl AxisScale {
    /// An axis that maps `max` exactly to the top of the plot.
    ///
    /// Used when the caller already knows the ceiling. The grid is derived
    /// from it; the ceiling itself is not rounded.
    fn exact(max: f32) -> Self {
        let max = if max > 0.0 { max } else { 1.0 };
        let step = max / AXIS_INTERVALS as f32;

        Self {
            max,
            step,
            decimals: decimals_for_step(step),
        }
    }

    /// A scale rounded up to a 1/2/2.5/5 × 10ⁿ step.
    ///
    /// The peak gets a little headroom, then the smallest round step that
    /// keeps the plot at five or fewer intervals is chosen. That keeps labels
    /// stable and easy to read while the data moves.
    fn nice(peak: f32) -> Self {
        if !peak.is_finite() || peak <= 0.0 {
            return Self {
                max: 1.0,
                step: 0.25,
                decimals: 2,
            };
        }

        let target = peak * 1.05;
        let raw = target / AXIS_INTERVALS as f32;
        let magnitude = 10f32.powf(raw.log10().floor());
        let mut step = 10.0 * magnitude;

        for fraction in [1.0, 2.0, 2.5, 5.0, 10.0] {
            let candidate = fraction * magnitude;

            if (target / candidate).ceil() <= AXIS_INTERVALS as f32 {
                step = candidate;
                break;
            }
        }

        let max = (target / step).ceil() * step;

        Self {
            max: max.max(step),
            step,
            decimals: decimals_for_step(step),
        }
    }

    /// The number of intervals between the floor and the ceiling.
    fn intervals(self) -> usize {
        (self.max / self.step).round().max(1.0) as usize
    }

    /// The value of the `index`th grid line from the floor.
    fn value(self, index: usize) -> f32 {
        (index as f32 * self.step).min(self.max)
    }
}

/// The number of decimal places needed to print a step without losing detail.
fn decimals_for_step(step: f32) -> usize {
    for decimals in 0..=3 {
        let scaled = step * 10f32.powi(decimals as i32);

        if (scaled - scaled.round()).abs() < 1e-4 {
            return decimals;
        }
    }

    3
}

/// Formats one Y-axis value. Zero is left undecorated to save space.
fn format_axis_value(value: f32, decimals: usize) -> String {
    if value.abs() < f32::EPSILON {
        "0".to_string()
    } else {
        format!("{value:.decimals$}")
    }
}

/// A secondary line drawn over a sparkline.
pub struct Line {
    values: Vec<f32>,
    color: Color,
    width: f32,
}

impl Line {
    pub fn new(values: Vec<f32>, color: Color, width: f32) -> Self {
        Self {
            values,
            color,
            width,
        }
    }
}

/// A horizontal reference line, such as a healthy/slow threshold.
pub struct Reference {
    value: f32,
    color: Color,
    label: String,
}

impl Reference {
    pub fn new(value: f32, color: Color, label: impl Into<String>) -> Self {
        Self {
            value,
            color,
            label: label.into(),
        }
    }
}

/// Draws a series as a line graph, optionally filled underneath and optionally
/// layered with extra lines.
///
/// An empty or all-zero series still draws the axes and grid, so the panel
/// does not look broken before the first frame is recorded.
pub struct Sparkline {
    /// The primary series, oldest first.
    primary: Vec<f32>,
    /// Draw a filled area under the primary curve.
    fill_primary: bool,
    /// Additional lines, e.g. rolling average and min/max.
    extras: Vec<Line>,
    /// The value range and grid step used by the plot.
    axis: AxisScale,
    /// Unit used by the Y-axis labels (`ms`, `fps`, ...).
    axis_unit: &'static str,
    /// Time span shown by the X axis, when the graph has time samples.
    time_window_secs: Option<f32>,
    /// Healthy/slow threshold lines drawn across the plot.
    references: Vec<Reference>,
    color: Color,
}

impl Sparkline {
    pub fn new(values: Vec<f32>, max: f32, color: Color) -> Self {
        Self {
            primary: values,
            fill_primary: true,
            extras: Vec::new(),
            axis: AxisScale::exact(max),
            axis_unit: "",
            time_window_secs: None,
            references: Vec::new(),
            color,
        }
    }

    /// A sparkline whose ceiling is rounded to a readable value derived from
    /// the data, with a little headroom so the peak is not flush against the
    /// top edge.
    pub fn auto_scaled(values: Vec<f32>, color: Color) -> Self {
        let peak = values.iter().copied().fold(0.0f32, f32::max);

        Self::new(values, 1.0, color).with_axis_scale(AxisScale::nice(peak))
    }

    /// A primary series plus extra lines, auto-scaled across all of them.
    pub fn layered(primary: Vec<f32>, extras: Vec<Line>, fill_primary: bool, color: Color) -> Self {
        let peak = primary
            .iter()
            .copied()
            .chain(extras.iter().flat_map(|line| line.values.iter().copied()))
            .fold(0.0f32, f32::max);

        Self {
            primary,
            fill_primary,
            extras,
            axis: AxisScale::nice(peak),
            axis_unit: "",
            time_window_secs: None,
            references: Vec::new(),
            color,
        }
    }

    fn with_axis_scale(mut self, axis: AxisScale) -> Self {
        self.axis = axis;
        self
    }

    /// Rounds the ceiling up from `peak` instead of deriving it from the data.
    ///
    /// The panel uses this to keep a nearby healthy/slow threshold in view.
    pub fn with_ceiling(mut self, peak: f32) -> Self {
        self.axis = AxisScale::nice(peak);
        self
    }

    /// Adds a unit to the Y-axis labels and a time span to the X axis.
    pub fn with_axis(mut self, unit: &'static str, time_window_secs: f32) -> Self {
        self.axis_unit = unit;
        self.time_window_secs = Some(time_window_secs.max(0.0));
        self
    }

    /// Adds a horizontal reference line, skipped when it is outside the plot.
    pub fn with_reference(mut self, value: f32, color: Color, label: impl Into<String>) -> Self {
        self.references.push(Reference::new(value, color, label));
        self
    }

    /// The points of the primary polyline, in the given bounds.
    ///
    /// The coordinates are relative to the canvas origin, not the window: the
    /// canvas widget already translates the geometry by `bounds` position
    /// before drawing, so adding `bounds.x`/`bounds.y` here would offset the
    /// graph out of its viewport.
    ///
    /// Separated out so the mapping can be tested without a GPU.
    fn points(&self, bounds: Rectangle) -> Vec<Point> {
        points(&self.primary, bounds, self.axis.max)
    }
}

/// The drawable plot area inside the canvas, leaving room for axis labels.
fn plot_rectangle(bounds: Rectangle) -> Rectangle {
    Rectangle {
        x: AXIS_LEFT,
        y: AXIS_TOP,
        width: (bounds.width - AXIS_LEFT - AXIS_RIGHT).max(1.0),
        height: (bounds.height - AXIS_TOP - AXIS_BOTTOM).max(1.0),
    }
}

/// Maps `values` to canvas coordinates.
fn points(values: &[f32], bounds: Rectangle, max: f32) -> Vec<Point> {
    if values.is_empty() {
        return Vec::new();
    }

    let plot = plot_rectangle(bounds);
    let max = if max > 0.0 { max } else { 1.0 };
    let step = if values.len() > 1 {
        plot.width / (values.len() - 1) as f32
    } else {
        plot.width
    };

    values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let ratio = (value / max).clamp(0.0, 1.0);

            Point::new(
                plot.x + step * index as f32,
                // Canvas y grows downward, so the ratio is inverted.
                plot.y + plot.height * (1.0 - ratio),
            )
        })
        .collect()
}

/// The y coordinate of a reference line, or `None` when it is off the plot.
fn reference_y(value: f32, plot: Rectangle, axis: AxisScale) -> Option<f32> {
    if !value.is_finite() || value <= 0.0 || value > axis.max {
        return None;
    }

    Some(plot.y + plot.height * (1.0 - value / axis.max))
}

/// The polygon under a curve: down to the floor at the start, along the curve,
/// then down to the floor at the end.
fn area_polygon(points: &[Point], floor: f32) -> Vec<Point> {
    let (Some(first), Some(last)) = (points.first(), points.last()) else {
        return Vec::new();
    };

    let mut polygon = Vec::with_capacity(points.len() + 2);
    polygon.push(Point::new(first.x, floor));
    polygon.extend_from_slice(points);
    polygon.push(Point::new(last.x, floor));
    polygon
}

fn polygon_path(polygon: &[Point]) -> Path {
    Path::new(|builder| {
        let Some(first) = polygon.first() else {
            return;
        };

        builder.move_to(*first);
        for point in &polygon[1..] {
            builder.line_to(*point);
        }
        builder.close();
    })
}

fn line_path(points: &[Point]) -> Path {
    Path::new(|builder| {
        let Some(first) = points.first() else {
            return;
        };

        builder.move_to(*first);
        for point in &points[1..] {
            builder.line_to(*point);
        }
    })
}

impl Program<DebugMessage, iced_winit::core::Theme, IcedRenderer> for Sparkline {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &IcedRenderer,
        _theme: &iced_winit::core::Theme,
        bounds: Rectangle,
        _cursor: iced_winit::core::mouse::Cursor,
    ) -> Vec<canvas::Geometry<IcedRenderer>> {
        let plot = plot_rectangle(bounds);
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        // Axes and grid are drawn even before the first sample arrives, so an
        // empty graph still shows its scale.
        draw_grid(&mut frame, plot, self.axis);

        let primary = self.points(bounds);

        if primary.len() >= 2 {
            // Filled area under the primary curve, when requested.
            if self.fill_primary {
                frame.fill(
                    &polygon_path(&area_polygon(&primary, plot.y + plot.height)),
                    Color {
                        a: 0.18,
                        ..self.color
                    },
                );
            }

            for line in &self.extras {
                let points = points(&line.values, bounds, self.axis.max);

                if points.len() >= 2 {
                    frame.stroke(
                        &line_path(&points),
                        Stroke {
                            style: line.color.into(),
                            width: line.width,
                            ..Default::default()
                        },
                    );
                }
            }

            frame.stroke(
                &line_path(&primary),
                Stroke {
                    style: self.color.into(),
                    width: 1.5,
                    ..Default::default()
                },
            );

            if let Some(last) = primary.last().copied() {
                frame.fill(&Path::circle(last, 2.5), self.color);
            }
        }

        draw_references(&mut frame, plot, self.axis, &self.references);
        draw_axis_labels(
            &mut frame,
            plot,
            self.axis,
            self.axis_unit,
            self.time_window_secs,
        );

        vec![frame.into_geometry()]
    }
}

fn draw_grid(frame: &mut canvas::Frame<IcedRenderer>, plot: Rectangle, axis: AxisScale) {
    for index in 0..=axis.intervals() {
        let value = axis.value(index);
        let y = plot.y + plot.height * (1.0 - value / axis.max);

        frame.fill_rectangle(
            Point::new(plot.x, y),
            Size::new(plot.width, 1.0),
            GRID_COLOR,
        );
    }

    for index in 0..=TIME_INTERVALS {
        let x = plot.x + plot.width * index as f32 / TIME_INTERVALS as f32;

        frame.fill_rectangle(
            Point::new(x, plot.y),
            Size::new(1.0, plot.height),
            MINOR_GRID_COLOR,
        );
    }

    // Axis lines frame the plot and make the zero line unambiguous.
    frame.fill_rectangle(
        Point::new(plot.x - 1.0, plot.y),
        Size::new(1.0, plot.height),
        AXIS_COLOR,
    );
    frame.fill_rectangle(
        Point::new(plot.x, plot.y + plot.height),
        Size::new(plot.width, 1.0),
        AXIS_COLOR,
    );
}

fn draw_references(
    frame: &mut canvas::Frame<IcedRenderer>,
    plot: Rectangle,
    axis: AxisScale,
    references: &[Reference],
) {
    let mut above_count = 0usize;

    for reference in references {
        let Some(y) = reference_y(reference.value, plot, axis) else {
            // Keep thresholds above the view visible as a small annotation so
            // a healthy budget is not silently dropped.
            if reference.value.is_finite() && reference.value > axis.max {
                draw_label(
                    frame,
                    format!("↑ {}", reference.label),
                    Point::new(
                        plot.x + plot.width,
                        plot.y + 1.0 + above_count as f32 * (REFERENCE_LABEL_SIZE + 1.0),
                    ),
                    Color {
                        a: 0.9,
                        ..reference.color
                    },
                    REFERENCE_LABEL_SIZE,
                    text::Alignment::Right,
                    alignment::Vertical::Top,
                );
                above_count += 1;
            }

            continue;
        };

        frame.stroke(
            &Path::line(Point::new(plot.x, y), Point::new(plot.x + plot.width, y)),
            Stroke {
                style: Color {
                    a: reference.color.a * 0.6,
                    ..reference.color
                }
                .into(),
                width: 1.0,
                line_dash: LineDash {
                    segments: DASH_SEGMENTS,
                    offset: 0,
                },
                ..Default::default()
            },
        );

        let (label_y, align_y) = if y - REFERENCE_LABEL_SIZE < plot.y {
            (y + 2.0, alignment::Vertical::Top)
        } else {
            (y - 2.0, alignment::Vertical::Bottom)
        };

        draw_label(
            frame,
            reference.label.clone(),
            Point::new(plot.x + plot.width, label_y),
            Color {
                a: 0.9,
                ..reference.color
            },
            REFERENCE_LABEL_SIZE,
            text::Alignment::Right,
            align_y,
        );
    }
}

fn draw_axis_labels(
    frame: &mut canvas::Frame<IcedRenderer>,
    plot: Rectangle,
    axis: AxisScale,
    unit: &'static str,
    time_window_secs: Option<f32>,
) {
    for index in 0..=axis.intervals() {
        let value = axis.value(index);
        let y = plot.y + plot.height * (1.0 - value / axis.max);
        let label = if index == axis.intervals() && !unit.is_empty() {
            format!("{} {unit}", format_axis_value(value, axis.decimals))
        } else {
            format_axis_value(value, axis.decimals)
        };

        draw_label(
            frame,
            label,
            Point::new(plot.x - 6.0, y),
            AXIS_LABEL_COLOR,
            AXIS_LABEL_SIZE,
            text::Alignment::Right,
            alignment::Vertical::Center,
        );
    }

    let Some(window_secs) = time_window_secs else {
        return;
    };

    let y = plot.y + plot.height + 3.0;
    for (content, x, align_x) in [
        (format!("-{window_secs:.1}s"), plot.x, text::Alignment::Left),
        (
            format!("-{:.1}s", window_secs * 0.5),
            plot.x + plot.width * 0.5,
            text::Alignment::Center,
        ),
        (
            "now".to_string(),
            plot.x + plot.width,
            text::Alignment::Right,
        ),
    ] {
        draw_label(
            frame,
            content,
            Point::new(x, y),
            AXIS_LABEL_COLOR,
            AXIS_LABEL_SIZE,
            align_x,
            alignment::Vertical::Top,
        );
    }
}

fn draw_label(
    frame: &mut canvas::Frame<IcedRenderer>,
    content: String,
    position: Point,
    color: Color,
    size: f32,
    align_x: text::Alignment,
    align_y: alignment::Vertical,
) {
    frame.fill_text(canvas::Text {
        content,
        position,
        color,
        size: Pixels::from(size),
        line_height: text::LineHeight::default(),
        font: Font::DEFAULT,
        align_x,
        align_y,
        shaping: text::Shaping::Basic,
        wrapping: text::Wrapping::default(),
        ellipsis: text::Ellipsis::default(),
        max_width: f32::INFINITY,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bounds() -> Rectangle {
        Rectangle {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0,
        }
    }

    #[test]
    fn empty_series_has_no_points() {
        assert!(
            Sparkline::new(Vec::new(), 1.0, Color::WHITE)
                .points(bounds())
                .is_empty()
        );
    }

    #[test]
    fn the_plot_is_inset_to_leave_room_for_labels() {
        assert_eq!(
            plot_rectangle(bounds()),
            Rectangle {
                x: 44.0,
                y: 8.0,
                width: 50.0,
                height: 24.0,
            }
        );
    }

    #[test]
    fn values_map_across_the_bounds() {
        let points = Sparkline::new(vec![0.0, 0.5, 1.0], 1.0, Color::WHITE).points(bounds());

        assert_eq!(points.len(), 3);
        assert_eq!(points[0], Point::new(44.0, 32.0), "zero sits on the floor");
        assert_eq!(points[1], Point::new(69.0, 20.0), "half is half way up");
        assert_eq!(points[2], Point::new(94.0, 8.0), "max is the ceiling");
    }

    #[test]
    fn points_are_relative_to_the_canvas_origin() {
        // The canvas translates the geometry by its bounds position itself, so
        // a non-zero origin must not leak into the points; the axis gutters
        // still apply inside the canvas.
        let shifted = Rectangle {
            x: 120.0,
            y: 40.0,
            ..bounds()
        };

        let points = Sparkline::new(vec![0.0, 1.0], 1.0, Color::WHITE).points(shifted);

        assert_eq!(points, vec![Point::new(44.0, 32.0), Point::new(94.0, 8.0)]);
    }

    #[test]
    fn a_single_value_is_drawn_at_the_left_edge() {
        // Degenerate: there is no step to advance, so the only sample sits at
        // the start.
        let points = Sparkline::new(vec![1.0], 1.0, Color::WHITE).points(bounds());

        assert_eq!(points, vec![Point::new(44.0, 8.0)]);
    }

    #[test]
    fn values_above_the_ceiling_are_clamped() {
        let points = Sparkline::new(vec![2.0, -1.0], 1.0, Color::WHITE).points(bounds());

        assert_eq!(points[0].y, 8.0, "clamped to the ceiling");
        assert_eq!(points[1].y, 32.0, "clamped to the floor");
    }

    #[test]
    fn auto_scaling_leaves_headroom_above_the_peak() {
        let sparkline = Sparkline::auto_scaled(vec![1.0, 5.0], Color::WHITE);
        assert!(sparkline.axis.max > 5.0, "max was {}", sparkline.axis.max);

        // An all-zero series must not divide by zero.
        let flat = Sparkline::auto_scaled(vec![0.0, 0.0], Color::WHITE);
        assert!(flat.axis.max > 0.0);
        assert_eq!(flat.points(bounds())[0].y, 32.0);
    }

    #[test]
    fn layered_scaling_considers_every_series() {
        let sparkline = Sparkline::layered(
            vec![1.0],
            vec![
                Line::new(vec![2.0], Color::WHITE, 1.0),
                Line::new(vec![8.0], Color::WHITE, 1.0),
            ],
            false,
            Color::WHITE,
        );

        assert!(sparkline.axis.max > 8.0);
        assert!(!sparkline.fill_primary);
    }

    #[test]
    fn the_area_polygon_follows_the_curve() {
        let points = points(&[0.2, 1.0, 0.2], bounds(), 1.0);
        let polygon = area_polygon(&points, 32.0);

        assert_eq!(
            polygon,
            vec![
                Point::new(44.0, 32.0),
                Point::new(44.0, 27.2),
                Point::new(69.0, 8.0),
                Point::new(94.0, 27.2),
                Point::new(94.0, 32.0),
            ]
        );
    }

    #[test]
    fn nice_scales_use_round_steps() {
        let frame_ms = AxisScale::nice(3.02);
        assert_eq!(frame_ms.max, 4.0);
        assert_eq!(frame_ms.step, 1.0);
        assert_eq!(frame_ms.decimals, 0);
        assert_eq!(frame_ms.intervals(), 4);

        let fps = AxisScale::nice(342.0);
        assert_eq!(fps.max, 400.0);
        assert_eq!(fps.step, 100.0);
        assert_eq!(fps.decimals, 0);

        let gpu_ms = AxisScale::nice(0.27);
        assert!((gpu_ms.max - 0.3).abs() < 1e-5);
        assert!((gpu_ms.step - 0.1).abs() < 1e-6);
        assert_eq!(gpu_ms.decimals, 1);

        let flat = AxisScale::nice(0.0);
        assert_eq!(flat.max, 1.0);
    }

    #[test]
    fn axis_values_reach_the_ceiling() {
        let axis = AxisScale::nice(3.02);

        assert_eq!(axis.value(0), 0.0);
        assert!((axis.value(axis.intervals()) - axis.max).abs() < 1e-5);
    }

    #[test]
    fn axis_labels_keep_only_the_precision_they_need() {
        assert_eq!(decimals_for_step(1.0), 0);
        assert_eq!(decimals_for_step(0.5), 1);
        assert_eq!(decimals_for_step(0.25), 2);
        assert_eq!(decimals_for_step(0.1), 1);
        assert_eq!(decimals_for_step(100.0), 0);

        assert_eq!(format_axis_value(0.0, 2), "0");
        assert_eq!(format_axis_value(1.5, 1), "1.5");
        assert_eq!(format_axis_value(400.0, 0), "400");
    }

    #[test]
    fn references_outside_the_plot_are_skipped() {
        let axis = AxisScale::nice(10.0);
        let plot = plot_rectangle(bounds());

        assert_eq!(reference_y(0.0, plot, axis), None);
        assert_eq!(reference_y(-1.0, plot, axis), None);
        assert_eq!(reference_y(axis.max + 1.0, plot, axis), None);

        assert_eq!(reference_y(axis.max, plot, axis), Some(plot.y));
        assert_eq!(
            reference_y(axis.max * 0.5, plot, axis),
            Some(plot.y + plot.height * 0.5)
        );
    }
}

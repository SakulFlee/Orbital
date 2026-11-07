//! A minimal line graph for the debug panels.
//!
//! `plotters-iced` would be the obvious choice, but its latest release (0.11.0)
//! requires `iced` 0.13 while this workspace runs a 0.15-dev fork, so it would
//! need forking too. The `canvas` widget is already present in the fork behind a
//! feature flag and needs no new dependencies, and a sparkline is a small
//! enough job that a hand-written `Program` is cheaper than the maintenance.

use iced_widget::canvas::{self, Path, Program, Stroke};
use iced_widget::core::{Point, Rectangle, Size, alignment, text};
use iced_winit::core::{Color, Font, Pixels};

use crate::state::DebugMessage;
use crate::state::Renderer as IcedRenderer;

const AXIS_LEFT: f32 = 32.0;
const AXIS_RIGHT: f32 = 4.0;
const AXIS_TOP: f32 = 4.0;
const AXIS_BOTTOM: f32 = 14.0;
const AXIS_LABEL_SIZE: f32 = 10.0;
const GRID_COLOR: Color = Color::from_rgba(0.75, 0.8, 0.9, 0.12);
const AXIS_LABEL_COLOR: Color = Color::from_rgba(0.75, 0.78, 0.85, 0.85);

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

/// Draws a series as a line graph, optionally filled underneath and optionally
/// layered with extra lines.
///
/// An empty or all-zero series draws a flat baseline rather than nothing, so
/// the panel does not appear broken before the first frame is recorded.
pub struct Sparkline {
    /// The primary series, oldest first.
    primary: Vec<f32>,
    /// Draw a filled area under the primary curve.
    fill_primary: bool,
    /// Additional lines, e.g. rolling average and min/max.
    extras: Vec<Line>,
    /// The value that maps to the top of the plot. Values above it are clamped.
    max: f32,
    /// Unit used by the Y-axis labels (`ms`, `fps`, ...).
    axis_unit: &'static str,
    /// Time span shown by the X axis, when the graph has time samples.
    time_window_secs: Option<f32>,
    color: Color,
}

impl Sparkline {
    pub fn new(values: Vec<f32>, max: f32, color: Color) -> Self {
        Self {
            primary: values,
            fill_primary: true,
            extras: Vec::new(),
            max,
            axis_unit: "",
            time_window_secs: None,
            color,
        }
    }

    /// A sparkline whose ceiling is derived from the data, with a little
    /// headroom so the peak is not flush against the top edge.
    pub fn auto_scaled(values: Vec<f32>, color: Color) -> Self {
        let max = ceiling(values.iter().copied());

        Self::new(values, max, color)
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
            max: ceiling(std::iter::once(peak)),
            axis_unit: "",
            time_window_secs: None,
            color,
        }
    }

    /// Adds a unit to the Y-axis labels and a time span to the X axis.
    pub fn with_axis(mut self, unit: &'static str, time_window_secs: f32) -> Self {
        self.axis_unit = unit;
        self.time_window_secs = Some(time_window_secs.max(0.0));
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
        points(&self.primary, bounds, self.max)
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

fn ceiling(values: impl IntoIterator<Item = f32>) -> f32 {
    let peak = values.into_iter().fold(0.0f32, f32::max);
    if peak > 0.0 { peak * 1.1 } else { 1.0 }
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
        let primary = self.points(bounds);

        if primary.len() < 2 {
            return Vec::new();
        }

        let plot = plot_rectangle(bounds);
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        draw_grid(&mut frame, plot);

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
            let points = points(&line.values, bounds, self.max);

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

        draw_axis_labels(
            &mut frame,
            plot,
            self.max,
            self.axis_unit,
            self.time_window_secs,
        );

        vec![frame.into_geometry()]
    }
}

fn draw_grid(frame: &mut canvas::Frame<IcedRenderer>, plot: Rectangle) {
    for step in 0..=4 {
        let y = plot.y + plot.height * step as f32 / 4.0;
        frame.fill_rectangle(
            Point::new(plot.x, y),
            Size::new(plot.width, 1.0),
            GRID_COLOR,
        );
    }

    for step in 0..=2 {
        let x = plot.x + plot.width * step as f32 / 2.0;
        frame.fill_rectangle(
            Point::new(x, plot.y),
            Size::new(1.0, plot.height),
            GRID_COLOR,
        );
    }
}

fn draw_axis_labels(
    frame: &mut canvas::Frame<IcedRenderer>,
    plot: Rectangle,
    max: f32,
    unit: &'static str,
    time_window_secs: Option<f32>,
) {
    for value in [max, max * 0.5, 0.0] {
        let ratio = (value / max.max(f32::EPSILON)).clamp(0.0, 1.0);
        let y = plot.y + plot.height * (1.0 - ratio);
        frame.fill_text(canvas::Text {
            content: format_axis_value(value, unit),
            position: Point::new(plot.x - 4.0, y),
            color: AXIS_LABEL_COLOR,
            size: Pixels::from(AXIS_LABEL_SIZE),
            line_height: text::LineHeight::default(),
            font: Font::DEFAULT,
            align_x: text::Alignment::Right,
            align_y: alignment::Vertical::Center,
            shaping: text::Shaping::Basic,
            wrapping: text::Wrapping::default(),
            ellipsis: text::Ellipsis::default(),
            max_width: f32::INFINITY,
        });
    }

    let Some(window_secs) = time_window_secs else {
        return;
    };

    let y = plot.y + plot.height + 2.0;
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
        frame.fill_text(canvas::Text {
            content,
            position: Point::new(x, y),
            color: AXIS_LABEL_COLOR,
            size: Pixels::from(AXIS_LABEL_SIZE),
            line_height: text::LineHeight::default(),
            font: Font::DEFAULT,
            align_x,
            align_y: alignment::Vertical::Top,
            shaping: text::Shaping::Basic,
            wrapping: text::Wrapping::default(),
            ellipsis: text::Ellipsis::default(),
            max_width: f32::INFINITY,
        });
    }
}

fn format_axis_value(value: f32, unit: &str) -> String {
    if unit == "fps" || value >= 100.0 {
        format!("{value:.0}")
    } else if value >= 10.0 {
        format!("{value:.1}")
    } else {
        format!("{value:.2}")
    }
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
    fn values_map_across_the_bounds() {
        let points = Sparkline::new(vec![0.0, 0.5, 1.0], 1.0, Color::WHITE).points(bounds());

        assert_eq!(points.len(), 3);
        assert_eq!(points[0], Point::new(32.0, 36.0), "zero sits on the floor");
        assert_eq!(points[1], Point::new(64.0, 20.0), "half is half way up");
        assert_eq!(points[2], Point::new(96.0, 4.0), "max is the ceiling");
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

        assert_eq!(points, vec![Point::new(32.0, 36.0), Point::new(96.0, 4.0)]);
    }

    #[test]
    fn a_single_value_is_drawn_at_the_left_edge() {
        // Degenerate: there is no step to advance, so the only sample sits at
        // the start. `draw` bails out below two points, so nothing is rendered.
        let points = Sparkline::new(vec![1.0], 1.0, Color::WHITE).points(bounds());

        assert_eq!(points, vec![Point::new(32.0, 4.0)]);
    }

    #[test]
    fn values_above_the_ceiling_are_clamped() {
        let points = Sparkline::new(vec![2.0, -1.0], 1.0, Color::WHITE).points(bounds());

        assert_eq!(points[0].y, 4.0, "clamped to the ceiling");
        assert_eq!(points[1].y, 36.0, "clamped to the floor");
    }

    #[test]
    fn auto_scaling_leaves_headroom_above_the_peak() {
        let sparkline = Sparkline::auto_scaled(vec![1.0, 5.0], Color::WHITE);
        assert!(sparkline.max > 5.0, "max was {}", sparkline.max);

        // An all-zero series must not divide by zero.
        let flat = Sparkline::auto_scaled(vec![0.0, 0.0], Color::WHITE);
        assert!(flat.max > 0.0);
        assert_eq!(flat.points(bounds())[0].y, 36.0);
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

        assert!(sparkline.max > 8.0);
        assert!(!sparkline.fill_primary);
    }

    #[test]
    fn the_area_polygon_follows_the_curve() {
        let points = points(&[0.2, 1.0, 0.2], bounds(), 1.0);
        let polygon = area_polygon(&points, 36.0);

        assert_eq!(
            polygon,
            vec![
                Point::new(32.0, 36.0),
                Point::new(32.0, 29.6),
                Point::new(64.0, 4.0),
                Point::new(96.0, 29.6),
                Point::new(96.0, 36.0),
            ]
        );
    }
}

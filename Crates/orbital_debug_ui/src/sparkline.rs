//! A minimal line graph for the debug panels.
//!
//! `plotters-iced` would be the obvious choice, but its latest release (0.11.0)
//! requires `iced` 0.13 while this workspace runs a 0.15-dev fork, so it would
//! need forking too. The `canvas` widget is already present in the fork behind a
//! feature flag and needs no new dependencies, and a sparkline is a small
//! enough job that a hand-written `Program` is cheaper than the maintenance.

use iced_widget::canvas::{self, Path, Program, Stroke};
use iced_widget::core::{Point, Rectangle};
use iced_winit::core::Color;

use crate::state::DebugMessage;
use crate::state::Renderer as IcedRenderer;

/// Draws a series as a filled-area line graph, optionally layered with a
/// rolling average and a min/max band, scaled to `max`.
///
/// An empty or all-zero series draws a flat baseline rather than nothing, so
/// the panel does not appear broken before the first frame is recorded.
pub struct Sparkline {
    /// The primary series, oldest first. Drawn with a filled area underneath.
    primary: Vec<f32>,
    /// An optional secondary line, e.g. the rolling average.
    average: Option<Vec<f32>>,
    /// An optional `(min, max)` envelope, drawn as a translucent band.
    band: Option<(Vec<f32>, Vec<f32>)>,
    /// The value that maps to the top of the plot. Values above it are clamped.
    max: f32,
    color: Color,
}

impl Sparkline {
    pub fn new(values: Vec<f32>, max: f32, color: Color) -> Self {
        Self {
            primary: values,
            average: None,
            band: None,
            max,
            color,
        }
    }

    /// A sparkline whose ceiling is derived from the data, with a little
    /// headroom so the peak is not flush against the top edge.
    pub fn auto_scaled(values: Vec<f32>, color: Color) -> Self {
        let max = ceiling(values.iter().copied());

        Self::new(values, max, color)
    }

    /// A sparkline with a primary series, an optional rolling average and an
    /// optional `(min, max)` band. The ceiling is derived from every series.
    pub fn layered(
        primary: Vec<f32>,
        average: Option<Vec<f32>>,
        band: Option<(Vec<f32>, Vec<f32>)>,
        color: Color,
    ) -> Self {
        let peak = primary
            .iter()
            .copied()
            .chain(average.iter().flatten().copied())
            .chain(
                band.iter()
                    .flat_map(|(min, max)| min.iter().chain(max.iter()).copied()),
            )
            .fold(0.0f32, f32::max);

        Self {
            primary,
            average,
            band,
            max: ceiling(std::iter::once(peak)),
            color,
        }
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

/// Maps `values` to canvas coordinates, relative to the canvas origin.
fn points(values: &[f32], bounds: Rectangle, max: f32) -> Vec<Point> {
    if values.is_empty() {
        return Vec::new();
    }

    let max = if max > 0.0 { max } else { 1.0 };
    let step = if values.len() > 1 {
        bounds.width / (values.len() - 1) as f32
    } else {
        bounds.width
    };

    values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let ratio = (value / max).clamp(0.0, 1.0);

            Point::new(
                step * index as f32,
                // Canvas y grows downward, so the ratio is inverted.
                bounds.height * (1.0 - ratio),
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

/// The polygon between two equally long curves: upper left-to-right, then
/// lower right-to-left.
fn band_polygon(lower: &[Point], upper: &[Point]) -> Vec<Point> {
    if lower.len() < 2 || lower.len() != upper.len() {
        return Vec::new();
    }

    let mut polygon = Vec::with_capacity(lower.len() + upper.len());
    polygon.extend_from_slice(upper);
    polygon.extend(lower.iter().rev().copied());
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

        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let floor = bounds.height;

        // Min/max envelope first, so both lines sit on top of it.
        if let Some((min, max)) = &self.band {
            let lower = points(min, bounds, self.max);
            let upper = points(max, bounds, self.max);
            let polygon = band_polygon(&lower, &upper);

            if !polygon.is_empty() {
                frame.fill(
                    &polygon_path(&polygon),
                    Color {
                        a: 0.10,
                        ..self.color
                    },
                );
            }
        }

        // Filled area under the primary curve, then the curve itself.
        frame.fill(
            &polygon_path(&area_polygon(&primary, floor)),
            Color {
                a: 0.18,
                ..self.color
            },
        );

        if let Some(average) = &self.average {
            let average = points(average, bounds, self.max);

            if average.len() >= 2 {
                frame.stroke(
                    &line_path(&average),
                    Stroke {
                        style: Color {
                            a: 0.65,
                            ..self.color
                        }
                        .into(),
                        width: 1.0,
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

        vec![frame.into_geometry()]
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
        assert_eq!(points[0], Point::new(0.0, 50.0), "zero sits on the floor");
        assert_eq!(points[1], Point::new(50.0, 25.0), "half is half way up");
        assert_eq!(points[2], Point::new(100.0, 0.0), "max is the ceiling");
    }

    #[test]
    fn points_are_relative_to_the_canvas_origin() {
        // The canvas translates the geometry by its bounds position itself, so
        // a non-zero origin must not leak into the points.
        let shifted = Rectangle {
            x: 120.0,
            y: 40.0,
            ..bounds()
        };

        let points = Sparkline::new(vec![0.0, 1.0], 1.0, Color::WHITE).points(shifted);

        assert_eq!(points, vec![Point::new(0.0, 50.0), Point::new(100.0, 0.0)]);
    }

    #[test]
    fn a_single_value_is_drawn_at_the_left_edge() {
        // Degenerate: there is no step to advance, so the only sample sits at
        // the start. `draw` bails out below two points, so nothing is rendered.
        let points = Sparkline::new(vec![1.0], 1.0, Color::WHITE).points(bounds());

        assert_eq!(points, vec![Point::new(0.0, 0.0)]);
    }

    #[test]
    fn values_above_the_ceiling_are_clamped() {
        let points = Sparkline::new(vec![2.0, -1.0], 1.0, Color::WHITE).points(bounds());

        assert_eq!(points[0].y, 0.0, "clamped to the ceiling");
        assert_eq!(points[1].y, 50.0, "clamped to the floor");
    }

    #[test]
    fn auto_scaling_leaves_headroom_above_the_peak() {
        let sparkline = Sparkline::auto_scaled(vec![1.0, 5.0], Color::WHITE);
        assert!(sparkline.max > 5.0, "max was {}", sparkline.max);

        // An all-zero series must not divide by zero.
        let flat = Sparkline::auto_scaled(vec![0.0, 0.0], Color::WHITE);
        assert!(flat.max > 0.0);
        assert_eq!(flat.points(bounds())[0].y, 50.0);
    }

    #[test]
    fn layered_scaling_considers_every_series() {
        let sparkline = Sparkline::layered(
            vec![1.0],
            Some(vec![2.0]),
            Some((vec![0.5], vec![8.0])),
            Color::WHITE,
        );

        assert!(sparkline.max > 8.0);
    }

    #[test]
    fn the_area_polygon_follows_the_curve() {
        // The old fill was a trapezoid from the first point to the last one,
        // which ignored the peak in the middle.
        let points = points(&[0.2, 1.0, 0.2], bounds(), 1.0);
        let polygon = area_polygon(&points, bounds().height);

        assert_eq!(
            polygon,
            vec![
                Point::new(0.0, 50.0),
                Point::new(0.0, 40.0),
                Point::new(50.0, 0.0),
                Point::new(100.0, 40.0),
                Point::new(100.0, 50.0),
            ]
        );
    }

    #[test]
    fn the_band_polygon_wraps_upper_then_lower() {
        let lower = points(&[0.2, 0.2], bounds(), 1.0);
        let upper = points(&[0.8, 0.8], bounds(), 1.0);
        let polygon = band_polygon(&lower, &upper);

        assert_eq!(polygon.len(), 4);
        assert_eq!(polygon[0], upper[0]);
        assert_eq!(polygon[1], upper[1]);
        assert_eq!(polygon[2], lower[1]);
        assert_eq!(polygon[3], lower[0]);
    }
}

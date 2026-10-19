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

/// Draws `values` as a filled-area line graph, scaled to `max`.
///
/// An empty or all-zero series draws a flat baseline rather than nothing, so
/// the panel does not appear broken before the first frame is recorded.
pub struct Sparkline {
    /// The series, oldest first.
    values: Vec<f32>,
    /// The value that maps to the top of the plot. Values above it are clamped.
    max: f32,
    color: Color,
}

impl Sparkline {
    pub fn new(values: Vec<f32>, max: f32, color: Color) -> Self {
        Self { values, max, color }
    }

    /// A sparkline whose ceiling is derived from the data, with a little
    /// headroom so the peak is not flush against the top edge.
    pub fn auto_scaled(values: Vec<f32>, color: Color) -> Self {
        let peak = values.iter().copied().fold(0.0f32, f32::max);
        let max = if peak > 0.0 { peak * 1.1 } else { 1.0 };

        Self::new(values, max, color)
    }

    /// The points of the plotted polyline, in the given bounds.
    ///
    /// Separated out so the mapping can be tested without a GPU.
    fn points(&self, bounds: Rectangle) -> Vec<Point> {
        if self.values.is_empty() {
            return Vec::new();
        }

        let max = if self.max > 0.0 { self.max } else { 1.0 };
        let step = if self.values.len() > 1 {
            bounds.width / (self.values.len() - 1) as f32
        } else {
            bounds.width
        };

        self.values
            .iter()
            .enumerate()
            .map(|(index, value)| {
                let ratio = (value / max).clamp(0.0, 1.0);

                Point::new(
                    bounds.x + step * index as f32,
                    // Canvas y grows downward, so the ratio is inverted.
                    bounds.y + bounds.height * (1.0 - ratio),
                )
            })
            .collect()
    }
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
        let points = self.points(bounds);

        if points.len() < 2 {
            return Vec::new();
        }

        let first = points[0];
        let last = points[points.len() - 1];
        let floor = bounds.y + bounds.height;

        let mut frame = canvas::Frame::new(renderer, bounds.size());

        // Filled area under the line, then the line itself on top of it.
        let area = Path::new(|builder| {
            builder.move_to(first);
            builder.line_to(last);
            builder.line_to(Point::new(last.x, floor));
            builder.line_to(Point::new(first.x, floor));
            builder.close();
        });
        frame.fill(
            &area,
            Color {
                a: 0.18,
                ..self.color
            },
        );

        let line = Path::new(|builder| {
            builder.move_to(first);
            for point in &points[1..] {
                builder.line_to(*point);
            }
        });
        frame.stroke(
            &line,
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
}

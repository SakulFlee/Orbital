//! Panel showing frame rate and per-stage render timings.

use iced_widget::{canvas, container, row, text};
use iced_winit::core::{Color, Element, Length, Theme};

use orbital_ecs::World;
use orbital_ecs_bridge::{FpsStats, FrameTimings, TOTAL_STAGE_INDEX};

use crate::sparkline::Sparkline;
use crate::state::Renderer as IcedRenderer;
use crate::state::{DebugMessage, DebugUiState, PanelId};

const PANEL_WIDTH: f32 = 380.0;

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
            // One graph: whole-frame time over the retained history. The
            // per-stage breakdown below stays numeric on purpose — stacking 11
            // series would be unreadable at this size, and the table answers
            // "which stage" more directly.
            sections.push(crate::panels::caption(
                "frame time (ms, last 240 frames)",
                Color::from_rgb(0.5, 0.9, 0.6),
            ));
            sections.push(
                canvas::Canvas::new(Sparkline::auto_scaled(
                    timings.frame_ms().iter().copied().collect(),
                    Color::from_rgb(0.5, 0.9, 0.6),
                ))
                .width(Length::Fill)
                .height(56)
                .into(),
            );

            sections.push(
                row![
                    text("stage").size(11).color(Color::from_rgb(0.6, 0.6, 0.6)),
                    text("last").size(11).color(Color::from_rgb(0.6, 0.6, 0.6)),
                    text("avg").size(11).color(Color::from_rgb(0.6, 0.6, 0.6)),
                    text("min").size(11).color(Color::from_rgb(0.6, 0.6, 0.6)),
                    text("max").size(11).color(Color::from_rgb(0.6, 0.6, 0.6)),
                ]
                .spacing(8)
                .into(),
            );

            for sample in timings.samples() {
                sections.push(
                    row![
                        text(sample.name).size(12),
                        text(format_ms(sample.last_ms)).size(12),
                        text(format_ms(sample.avg_ms)).size(12),
                        text(format_ms(sample.min_ms)).size(12),
                        text(format_ms(sample.max_ms)).size(12),
                    ]
                    .spacing(8)
                    .into(),
                );
            }

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

    container(
        crate::panels::column_of(sections, 2.0)
            .padding(10)
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .width(PANEL_WIDTH)
    .into()
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
}

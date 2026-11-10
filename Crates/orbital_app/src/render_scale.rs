//! Internal render resolution and adaptive render-scale control.
//!
//! The surface keeps its native size; only the 3D scene targets are scaled.
//! Tonemapping and the anti-aliasing pass stretch the internal image back to
//! the surface, so 2D, UI, and text overlays stay at native resolution.

use cgmath::Vector2;

/// Smallest render scale the runtime accepts.
pub const MIN_RENDER_SCALE: f32 = 0.25;
/// Largest render scale the runtime accepts.
pub const MAX_RENDER_SCALE: f32 = 1.0;

/// Rounding granularity used by the adaptive controller.
const ADAPTIVE_SCALE_STEP: f32 = 0.05;
/// Number of timing windows a stable frame rate must hold before the scale is
/// raised again.
const ADAPTIVE_RAISE_WINDOWS: u8 = 4;

/// Converts a surface resolution and render scale into a positive render
/// resolution.
///
/// The result is rounded to whole pixels and clamped so a valid texture can
/// always be created.
pub fn scaled_resolution(surface: Vector2<u32>, scale: f32) -> Vector2<u32> {
    let scale = if scale.is_finite() {
        scale.clamp(MIN_RENDER_SCALE, MAX_RENDER_SCALE)
    } else {
        1.0
    };

    Vector2::new(
        ((surface.x as f32 * scale).round() as u32).max(1),
        ((surface.y as f32 * scale).round() as u32).max(1),
    )
}

/// Effective performance target after taking display refresh and an explicit
/// frame cap into account.
///
/// A target above the display refresh rate can never be reached, so clamping it
/// prevents the adaptive controller from shrinking the image forever. The same
/// applies to an explicit frame cap: the limiter intentionally holds the frame
/// rate below the configured target.
pub fn effective_target_fps(
    target_fps: u32,
    refresh_rate_mhz: Option<u32>,
    frame_cap: Option<u32>,
) -> u32 {
    let mut target = target_fps.max(1);

    if let Some(mhz) = refresh_rate_mhz.filter(|mhz| *mhz > 0) {
        let refresh_hz = ((mhz as f32 / 1000.0).round() as u32).max(1);
        target = target.min(refresh_hz);
    }

    if let Some(cap) = frame_cap.filter(|cap| *cap > 0) {
        target = target.min(cap);
    }

    target
}

/// Stateful adaptive render-scale controller.
///
/// The controller is intentionally conservative: it lowers the scale in the
/// first timing window that misses the target and only raises it after several
/// consecutive windows meet it. That avoids visible oscillation around the
/// target frame rate.
#[derive(Debug, Default, Clone, Copy)]
pub struct AdaptiveRenderScale {
    stable_windows: u8,
}

impl AdaptiveRenderScale {
    /// Returns the next render scale, or `None` when the current one is already
    /// appropriate.
    ///
    /// `fps` is the completed one-second frame count. Scales are clamped to the
    /// supplied range and quantized to [`ADAPTIVE_SCALE_STEP`] so the reported
    /// value stays readable.
    pub fn update(
        &mut self,
        current_scale: f32,
        fps: u32,
        target_fps: u32,
        min_scale: f32,
        max_scale: f32,
    ) -> Option<f32> {
        if fps == 0 {
            return None;
        }

        let min_scale = min_scale.clamp(MIN_RENDER_SCALE, MAX_RENDER_SCALE);
        let max_scale = max_scale.clamp(min_scale, MAX_RENDER_SCALE);
        let current_scale = if current_scale.is_finite() {
            current_scale.clamp(min_scale, max_scale)
        } else {
            max_scale
        };
        let target_fps = target_fps.max(1);

        // Integer threshold: 95% of the target, rounded up, so a 60 FPS target
        // starts reacting at 57 FPS rather than bouncing on a single bad frame.
        let deficient_below = (target_fps * 95).div_ceil(100);

        if fps < deficient_below {
            self.stable_windows = 0;
            let next = quantize(current_scale - ADAPTIVE_SCALE_STEP, min_scale, max_scale);
            return (next < current_scale - 0.001).then_some(next);
        }

        if fps >= target_fps {
            self.stable_windows = self.stable_windows.saturating_add(1);
            if self.stable_windows >= ADAPTIVE_RAISE_WINDOWS {
                self.stable_windows = 0;
                let next = quantize(current_scale + ADAPTIVE_SCALE_STEP, min_scale, max_scale);
                return (next > current_scale + 0.001).then_some(next);
            }
        } else {
            self.stable_windows = 0;
        }

        None
    }
}

fn quantize(value: f32, min: f32, max: f32) -> f32 {
    let steps = (value / ADAPTIVE_SCALE_STEP).round();
    (steps * ADAPTIVE_SCALE_STEP).clamp(min, max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scaled_resolution_rounds_and_clamps() {
        assert_eq!(
            scaled_resolution(Vector2::new(2560, 1600), 0.5),
            Vector2::new(1280, 800)
        );
        assert_eq!(
            scaled_resolution(Vector2::new(1220, 2582), 0.75),
            Vector2::new(915, 1937)
        );
        assert_eq!(
            scaled_resolution(Vector2::new(10, 10), 0.01),
            Vector2::new(3, 3)
        );
        assert_eq!(
            scaled_resolution(Vector2::new(10, 10), f32::NAN),
            Vector2::new(10, 10)
        );
        assert_eq!(
            scaled_resolution(Vector2::new(0, 0), 1.0),
            Vector2::new(1, 1)
        );
    }

    #[test]
    fn effective_target_is_capped_by_refresh_and_frame_cap() {
        assert_eq!(effective_target_fps(60, Some(48_000), None), 48);
        assert_eq!(effective_target_fps(60, Some(144_000), None), 60);
        assert_eq!(effective_target_fps(60, Some(0), Some(30)), 30);
        assert_eq!(effective_target_fps(60, None, Some(120)), 60);
        assert_eq!(effective_target_fps(0, None, None), 1);
    }

    #[test]
    fn adaptive_scale_drops_immediately_when_below_target() {
        let mut controller = AdaptiveRenderScale::default();

        assert_eq!(
            controller.update(1.0, 45, 60, 0.5, 1.0),
            Some(0.95),
            "a missed target lowers the scale"
        );
        assert_eq!(
            controller.update(0.5, 10, 60, 0.5, 1.0),
            None,
            "the minimum scale is a hard floor"
        );
    }

    #[test]
    fn adaptive_scale_raises_only_after_stable_windows() {
        let mut controller = AdaptiveRenderScale::default();

        for _ in 0..ADAPTIVE_RAISE_WINDOWS - 1 {
            assert_eq!(controller.update(0.5, 60, 60, 0.5, 1.0), None);
        }

        assert_eq!(
            controller.update(0.5, 60, 60, 0.5, 1.0),
            Some(0.55),
            "the scale rises after a stable window streak"
        );
        assert_eq!(
            controller.update(1.0, 60, 60, 0.5, 1.0),
            None,
            "the maximum scale is a hard ceiling"
        );
    }

    #[test]
    fn adaptive_scale_ignores_empty_windows() {
        let mut controller = AdaptiveRenderScale::default();
        assert_eq!(controller.update(0.5, 0, 60, 0.5, 1.0), None);
    }
}

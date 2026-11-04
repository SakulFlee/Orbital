//! Optional frame-rate limiter for the winit event loop.
//!
//! The limiter sleeps relative to the start of the current redraw, so time
//! spent in present and CPU work counts toward the target period. If a frame
//! runs past its deadline, the next frame starts immediately instead of
//! accumulating a catch-up backlog.

use std::time::{Duration, Instant};

/// Frame limiter state.
#[derive(Debug, Clone, Copy)]
pub struct FrameLimiter {
    /// Below this remaining time, spin instead of sleeping for a more precise
    /// wake-up.
    spin_threshold: Duration,
}

impl Default for FrameLimiter {
    fn default() -> Self {
        Self {
            spin_threshold: Duration::from_millis(1),
        }
    }
}

impl FrameLimiter {
    /// Waits until the target period has elapsed since `frame_start`.
    ///
    /// VSync already paces presentation, so an explicit limit is ignored when
    /// VSync is enabled. `None` and `Some(0)` also disable the limiter.
    pub fn wait(&self, frame_start: Instant, fps_limit: Option<u32>, vsync_enabled: bool) {
        let Some(period) = target_period(fps_limit, vsync_enabled) else {
            return;
        };

        let deadline = frame_start + period;
        let now = Instant::now();
        if now >= deadline {
            return;
        }

        let remaining = deadline.saturating_duration_since(now);
        if remaining > self.spin_threshold {
            std::thread::sleep(remaining - self.spin_threshold);
        }

        while Instant::now() < deadline {
            std::hint::spin_loop();
        }
    }
}

/// Target frame period, or `None` when limiting is disabled or overridden by
/// VSync.
fn target_period(fps_limit: Option<u32>, vsync_enabled: bool) -> Option<Duration> {
    if vsync_enabled {
        return None;
    }

    fps_limit
        .filter(|fps| *fps > 0)
        .map(|fps| Duration::from_secs_f64(1.0 / fps as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_period_uses_the_requested_rate() {
        assert_eq!(
            target_period(Some(50), false),
            Some(Duration::from_millis(20))
        );
        assert_eq!(
            target_period(Some(144), false),
            Some(Duration::from_secs_f64(1.0 / 144.0))
        );
    }

    #[test]
    fn vsync_and_invalid_limits_disable_the_limiter() {
        assert_eq!(target_period(Some(60), true), None);
        assert_eq!(target_period(None, false), None);
        assert_eq!(target_period(Some(0), false), None);
    }

    #[test]
    fn expired_deadline_does_not_sleep() {
        let limiter = FrameLimiter::default();
        let start = Instant::now() - Duration::from_millis(200);
        let before = Instant::now();

        limiter.wait(start, Some(10), false);

        assert!(before.elapsed() < Duration::from_millis(20));
    }
}

use std::time::Instant;

pub struct Timer {
    last_time: Instant,
    fps: u64,
    delta_time: f64,
    cycle_delta_time: f64,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            last_time: Instant::now(),
            fps: 0u64,
            delta_time: 0f64,
            cycle_delta_time: 0f64,
        }
    }

    pub fn tick(&mut self) -> (f64, Option<(f64, u64)>) {
        self.tick_at(Instant::now())
    }

    fn tick_at(&mut self, now: Instant) -> (f64, Option<(f64, u64)>) {
        let elapsed = now.duration_since(self.last_time);
        self.last_time = now;

        let raw_elapsed = elapsed.as_secs_f64();
        self.delta_time = raw_elapsed.clamp(0.0, 1.0);

        // Frame delta is clamped for gameplay, but the FPS window must use the
        // real elapsed time. Otherwise a long stall silently disappears from
        // the reported FPS and window duration.
        self.cycle_delta_time += raw_elapsed;
        self.fps += 1;

        let cycle_part = if self.cycle_delta_time >= 1.0 {
            let output = Some((self.cycle_delta_time, self.fps));

            if raw_elapsed >= 1.0 {
                // A single frame already exceeded a full window. Close the
                // window instead of replaying the stall as several catch-up
                // windows on the following frames.
                self.cycle_delta_time = 0.0;
            } else {
                self.cycle_delta_time -= 1.0;
            }
            self.fps = 0;

            output
        } else {
            None
        };

        (self.delta_time, cycle_part)
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn long_frame_reports_its_full_duration() {
        let mut timer = Timer::new();
        let now = Instant::now();
        timer.last_time = now - Duration::from_millis(3_000);

        let (delta, cycle) = timer.tick_at(now);
        let (total_delta, fps) = cycle.expect("a long frame closes the FPS window");

        assert_eq!(delta, 1.0, "gameplay delta remains clamped");
        assert!((total_delta - 3.0).abs() < 1e-6);
        assert_eq!(fps, 1);
        assert_eq!(timer.cycle_delta_time, 0.0, "no catch-up backlog");
    }

    #[test]
    fn normal_window_keeps_its_remainder() {
        let mut timer = Timer::new();
        let start = Instant::now();
        timer.last_time = start;

        let (_, first) = timer.tick_at(start + Duration::from_millis(600));
        assert!(first.is_none());
        assert!((timer.cycle_delta_time - 0.6).abs() < 1e-6);

        let (_, second) = timer.tick_at(start + Duration::from_millis(1_200));
        let (total_delta, fps) = second.expect("the accumulated window closes");

        assert!((total_delta - 1.2).abs() < 1e-6);
        assert_eq!(fps, 2);
        assert!((timer.cycle_delta_time - 0.2).abs() < 1e-6);
    }

    #[test]
    fn zero_elapsed_tick_does_not_close_the_window() {
        let mut timer = Timer::new();
        let now = Instant::now();
        timer.last_time = now;

        let (delta, cycle) = timer.tick_at(now);
        assert_eq!(delta, 0.0);
        assert!(cycle.is_none());
    }
}

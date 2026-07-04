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
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_time);
        self.last_time = now;

        self.delta_time = elapsed.as_secs_f64().clamp(0.0, 1.0);

        self.cycle_delta_time += self.delta_time;
        self.fps += 1;

        let cycle_part = if self.cycle_delta_time >= 1.0 {
            let output = Some((self.cycle_delta_time, self.fps));

            self.cycle_delta_time -= 1.0;
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

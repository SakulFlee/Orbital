use std::time::Instant;

pub struct AppCycleCounter {
    last_cycle_time: Instant,
    current_cycle_count: u32,
    delta_time: f64,
    cycles_per_second: u32,
}

impl AppCycleCounter {
    pub fn reset(&mut self) {
        self.last_cycle_time = Instant::now();
        self.current_cycle_count = 0;
        self.delta_time = 0.0;
        self.cycles_per_second = 0;
    }

    pub fn tick(&mut self) -> Option<(f64, u32)> {
        // Take now time
        let now = Instant::now();

        // Increase current cycle count
        self.current_cycle_count += 1;

        // Calculate duration since last cycle
        let delta_duration = now.duration_since(self.last_cycle_time);

        // Increment delta time by amount
        self.delta_time += delta_duration.as_secs_f64();

        // Update last cycle time
        self.last_cycle_time = now;

        // If a second in delta time past
        if self.delta_time >= 1.0 {
            // Reset cycle count and update cycles per second
            self.cycles_per_second = self.current_cycle_count;
            self.current_cycle_count = 0;

            // Decrease delta time
            let current_delta_time = self.delta_time;
            self.delta_time -= 1.0;

            return Some((current_delta_time, self.cycles_per_second));
        }

        return None;
    }
}

impl Default for AppCycleCounter {
    fn default() -> Self {
        Self {
            last_cycle_time: Instant::now(),
            current_cycle_count: 0,
            delta_time: 0.0,
            cycles_per_second: 0,
        }
    }
}

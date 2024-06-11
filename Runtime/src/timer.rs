use std::time::Instant;

pub struct Timer {
    last_time: Instant,
    current_cycle_count: u64,
    cycle_delta_time: f64,
    current_delta_time: f64,
    last_fps: u64,
    last_delta_time: f64,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            last_time: Instant::now(),
            current_cycle_count: 0u64,
            cycle_delta_time: 0f64,
            current_delta_time: 0f64,
            last_fps: 0u64,
            last_delta_time: 0f64,
        }
    }

    pub fn tick(&mut self) -> Option<(f64, u64)> {
        // Take a snapshot and reassign our time
        let elapsed = self.last_time.elapsed();
        self.last_time = Instant::now();

        // Convert our snapshot into elapsed seconds, increase current delta
        // time and increment cycle count
        self.cycle_delta_time = elapsed.as_secs_f64();
        self.current_delta_time += self.cycle_delta_time;
        self.current_cycle_count += 1;

        // If current delta time is more than a second, reset cycle and return
        if self.current_delta_time >= 1.0 {
            // Make the result FIRST!
            // We are resetting the timer below!
            self.last_fps = self.current_cycle_count();
            self.last_delta_time = self.current_delta_time();
            let output = Some((self.last_delta_time, self.last_fps));

            self.current_cycle_count = 0;
            self.current_delta_time -= 1.0;
            if self.current_delta_time < 0.0 {
                self.current_delta_time = 0.0;
            }

            return output;
        }

        None
    }

    pub fn current_cycle_count(&self) -> u64 {
        self.current_cycle_count
    }

    pub fn cycle_delta_time(&self) -> f64 {
        self.cycle_delta_time
    }

    pub fn current_delta_time(&self) -> f64 {
        self.current_delta_time
    }

    pub fn last_fps(&self) -> u64 {
        self.last_fps
    }

    pub fn last_delta_time(&self) -> f64 {
        self.last_delta_time
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

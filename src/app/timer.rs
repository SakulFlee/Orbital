use std::time::Instant;

pub struct Timer {
    last_time: Instant,
    current_cycle_count: u64,
    current_delta_time: f64,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            last_time: Instant::now(),
            current_cycle_count: 0,
            current_delta_time: 0.0,
        }
    }

    pub fn tick(&mut self) -> Option<(f64, u64)> {
        let elapsed = self.last_time.elapsed();
        self.last_time = Instant::now();

        self.current_delta_time += elapsed.as_secs_f64();
        self.current_cycle_count += 1;

        if self.current_delta_time >= 1.0 {
            let output = Some((
                self.get_current_delta_time(),
                self.get_current_cycle_count(),
            ));

            self.current_cycle_count = 0;
            self.current_delta_time -= 1.0;

            return output;
        }

        None
    }

    pub fn get_current_cycle_count(&self) -> u64 {
        self.current_cycle_count
    }

    pub fn get_current_delta_time(&self) -> f64 {
        self.current_delta_time
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

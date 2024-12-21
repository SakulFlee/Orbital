use std::time::Instant;

pub struct Timer {
    /// Last time when tick was called.
    /// Used to calculate the current delta.
    last_time: Instant,
    /// FPS = UPS  
    /// (Frames Per Second == Updates Per Second)
    fps: u64,
    /// The time that has passed since the last tick.  
    /// I.e. the time passed since the last update cycle and frame.
    delta_time: f64,
    /// The total time that has passed during this whole cycle.
    /// Each cycle should last at least a single second, but it can be longer in case of e.g. lag.
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

    /// Will perform a few calculations in succession to accurately calculate a delta time, to be used in e.g. updating, and fps reading.
    /// Must be called each update cycle, otherwise this will be inaccurate.
    ///
    /// # Delta time vs. Cycle delta time
    /// Delta time is the time passed since the last tick (and thus update/frame).
    /// Cycle delta time is the time passed since the last cycle.
    ///
    /// Cycle delta time is an accumulation (summarization) of delta times, over at least one second, possibly longer in case of e.g. lag.
    ///
    /// Thus, delta time should be used to e.g. update between frames.  
    /// Cycle delta time should be used as a metric of FPS stability.
    ///
    /// # Returns
    /// Returns two (*three) things as a tuple:
    /// 1. The current delta time, will always be returned.
    /// 2. `Some(fps, cycle delta time)`, if a cycle has concluded.
    ///    `None`, otherwise.
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

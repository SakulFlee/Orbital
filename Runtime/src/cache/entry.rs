use std::time::{Duration, Instant};

pub struct CacheEntry<Value> {
    timer: Instant,
    value: Value,
}

impl<Value> CacheEntry<Value> {
    pub fn new(value: Value) -> Self {
        Self {
            timer: Instant::now(),
            value,
        }
    }

    pub fn reset_timer(&mut self) {
        self.timer = Instant::now();
    }

    pub fn elapsed(&self) -> Duration {
        self.timer.elapsed()
    }

    pub fn value(&self) -> &Value {
        &self.value
    }

    pub fn value_mut(&mut self) -> &mut Value {
        &mut self.value
    }
}

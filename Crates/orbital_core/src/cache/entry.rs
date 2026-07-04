use std::{ops::Deref, sync::Arc, time::Instant};

#[derive(Debug)]
pub struct CacheEntry<Value> {
    inner: Arc<Value>,
    unused_since: Option<Instant>,
}

impl<Value> CacheEntry<Value> {
    pub fn new(value: Value) -> Self {
        Self {
            inner: Arc::new(value),
            unused_since: None,
        }
    }

    pub fn cleanup_check(&mut self) {
        if Arc::<Value>::strong_count(&self.inner) == 1 {
            self.unused_since = Some(Instant::now());
        } else {
            self.unused_since = None;
        }
    }

    pub fn inner(&self) -> &Value {
        &self.inner
    }

    pub fn clone_inner(&self) -> Arc<Value> {
        self.inner.clone()
    }

    pub fn unused_since(&self) -> Option<&Instant> {
        self.unused_since.as_ref()
    }
}

impl<Value> Deref for CacheEntry<Value> {
    type Target = Value;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

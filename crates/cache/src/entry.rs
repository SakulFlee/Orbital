use std::{ops::Deref, sync::Arc, time::Instant};

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

    /// Checks the _strong reference count_ of the inner `Arc` holding our `Value`.
    /// If said counter is more than 1, that means that the value is still in use somewhere.
    /// The counter can never be below zero (0), as we are holding the Arc ourselves here.
    /// > Note: We assume here, that anyone using the given resources, `clone`s said inner Arc and keeps it stored for as long as needed.
    ///
    /// If the counter is 1, we assume that the value is no longer in use and mark it as unused by setting a timer.
    /// The outer cache will periodically check if the entry is unused and remove it if it passes beyond a threshold.
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

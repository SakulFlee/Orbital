use std::{
    hash::Hash,
    ops::{Deref, DerefMut},
    time::Duration,
};

use hashbrown::HashMap;
use log::debug;

mod entry;
pub use entry::*;

#[derive(Debug)]
pub struct Cache<Key, Value>
where
    Key: Sized + Hash + PartialEq + Eq + Clone,
    Value: Sized,
{
    timeout: Duration,
    map: HashMap<Key, CacheEntry<Value>>,
}

impl<Key, Value> Default for Cache<Key, Value>
where
    Key: Sized + Hash + PartialEq + Eq + Clone,
    Value: Sized,
{
    fn default() -> Self {
        Self::new(Duration::from_secs(5))
    }
}

impl<Key, Value> Cache<Key, Value>
where
    Key: Sized + Hash + PartialEq + Eq + Clone,
    Value: Sized,
{
    pub fn new(timeout: Duration) -> Self {
        Self {
            timeout,
            map: HashMap::new(),
        }
    }

    pub fn cleanup(&mut self) {
        // Perform cleanup check first
        self.map.values_mut().for_each(CacheEntry::cleanup_check);

        #[cfg(debug_assertions)]
        let before = self.map.len();

        // Then remove anything past our threshold
        self.map
            .retain(|_, v| v.unused_since().is_none_or(|x| x.elapsed() <= self.timeout));

        #[cfg(debug_assertions)]
        {
            let after = self.map.len();
            debug!(
                "Cache cleanup: {before} -> {after} entries (before -> after)"
            );
        }
    }
}

impl<Key, Value> Deref for Cache<Key, Value>
where
    Key: Sized + Hash + PartialEq + Eq + Clone,
    Value: Sized,
{
    type Target = HashMap<Key, CacheEntry<Value>>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl<Key, Value> DerefMut for Cache<Key, Value>
where
    Key: Sized + Hash + PartialEq + Eq + Clone,
    Value: Sized,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}

use std::{hash::Hash, time::Duration};

use change::CacheChange;
use entry::CacheEntry;
use hashbrown::HashMap;
use log::debug;

pub mod change;
pub mod entry;

pub struct Cache<Key, Value>
where
    Key: Sized + Hash + PartialEq + Eq + Clone,
    Value: Sized,
{
    map: HashMap<Key, CacheEntry<Value>>,
}

impl<Key, Value> Default for Cache<Key, Value>
where
    Key: Sized + Hash + PartialEq + Eq + Clone,
    Value: Sized,
 {
    fn default() -> Self {
        Self::new()
    }
}

impl<Key, Value> Cache<Key, Value>
where
    Key: Sized + Hash + PartialEq + Eq + Clone,
    Value: Sized,
{
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Gets or add a `Value` given a `Key`.
    ///
    /// If the `Key` doesn't exist in the [Cache] yet, `func` will be invoked with the `Key` and is expected to generate the appropriate `Value`.
    ///
    /// If the `Key` does exist, it's [Cache] hit timer will be reset and a reference to `Value` is returned.
    pub fn get_or_add<F>(&mut self, key: &Key, func: F) -> &Value
    where
        F: FnOnce(&Key) -> Value,
    {
        // Unfortunately needed, `u8` is cheap.
        type FakeError = u8;
        self.get_or_add_fallible(key, |k| Ok::<Value, FakeError>(func(k)))
            .unwrap()
    }

    /// Gets or add a `Value` given a `Key`.
    ///
    /// Same as [Self::get_or_add], but `func` returns a [Result].
    /// If the result of `func` is [Err], the [Err] will be returned.
    /// Otherwise, [Ok] will be returned.
    pub fn get_or_add_fallible<F, E>(&mut self, key: &Key, func: F) -> Result<&Value, E>
    where
        F: FnOnce(&Key) -> Result<Value, E>,
    {
        if !self.map.contains_key(key) {
            let value = func(key)?;
            let wrapped = CacheEntry::new(value);

            self.map.insert(key.clone(), wrapped);
            Ok(self.map.get(key).unwrap().value())
        } else {
            let wrapper = self.map.get_mut(key).unwrap();
            wrapper.reset_timer();
            Ok(wrapper.value())
        }
    }

    /// Gets or add a `Value` given a `Key`.
    ///
    /// If the `Key` doesn't exist in the [Cache] yet, `func` will be invoked with the `Key` and is expected to generate the appropriate `Value`.
    ///
    /// If the `Key` does exist, it's [Cache] hit timer will be reset and a mutable reference to `Value` is returned.
    pub fn get_or_add_mut<F>(&mut self, key: &Key, func: F) -> &mut Value
    where
        F: FnOnce(&Key) -> Value,
    {
        // Unfortunately needed, `u8` is cheap.
        type FakeError = u8;
        self.get_or_add_mut_fallible(key, |k| Ok::<Value, FakeError>(func(k)))
            .unwrap()
    }

    /// Gets or add a `Value` given a `Key`.
    ///
    /// Same as [Self::get_or_add_mut], but `func` returns a [Result].
    /// If the result of `func` is [Err], the [Err] will be returned.
    /// Otherwise, [Ok] will be returned.
    pub fn get_or_add_mut_fallible<F, E>(&mut self, key: &Key, func: F) -> Result<&mut Value, E>
    where
        F: FnOnce(&Key) -> Result<Value, E>,
    {
        if !self.map.contains_key(key) {
            let value = func(key)?;
            let wrapped = CacheEntry::new(value);

            self.map.insert(key.clone(), wrapped);
            Ok(self.map.get_mut(key).unwrap().value_mut())
        } else {
            let wrapper = self.map.get_mut(key).unwrap();
            wrapper.reset_timer();
            Ok(wrapper.value_mut())
        }
    }

    /// Runs a cleanup operation on the cache.
    /// Any value with a key that is longer or equal than [Duration] `retain_below` will be **removed**.
    /// This, effectively, should drop any expired cache values.
    ///
    /// Set the `retain_below` [Duration] to a proper value!
    /// Set it too low and your [Cache] will be ineffective.
    /// Set it too high and you may run into memory issues!
    /// > This depends on what `Value`'s you are actually storing!
    pub fn cleanup(&mut self, retain_below: Duration) -> CacheChange {
        let mut change = CacheChange::default();
        change.before = self.size();

        self.map.retain(|_k, v| {
            let elapsed = v.elapsed();
            debug!("Elapsed: {:?}", elapsed);
            let condition = elapsed < retain_below;
            debug!("Condition: {}", condition);
            condition
        });

        change.after = self.size();
        change
    }

    /// Used to rework the [Cache] by looping through all keys and re-making them with the given closure.
    ///
    /// This can be useful e.g. in the [Pipeline](crate::resources::realizations::Pipeline) [Cache] where each [Pipeline](crate::resources::realizations::Pipeline) needs to be remade/recompiled once the [TextureFormat](crate::wgpu::TextureFormat) changes.
    ///
    /// If [None] is returned by `func`, the value will be dropped.
    /// This should only be used in case of errors.
    pub fn rework<F>(&mut self, func: F)
    where
        F: Fn(&Key) -> Option<Value>,
    {
        let mut new_map = HashMap::new();

        self.map.iter().for_each(|(key, _)| {
            if let Some(new_value) = func(key) {
                let new_wrapper = CacheEntry::new(new_value);
                new_map.insert(key.clone(), new_wrapper);
            }
        });

        self.map = new_map;
    }

    pub fn size(&self) -> usize {
        self.map.len()
    }
}

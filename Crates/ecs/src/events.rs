use std::collections::VecDeque;
use std::marker::PhantomData;

/// A generic event queue for passing data between systems.
///
/// Events are sent during a frame and read by systems that care about them.
/// After all systems have had a chance to read, `clear()` should be called
/// to remove processed events.
///
/// # Example
/// ```ignore
/// // Define an event
/// #[derive(Debug)]
/// struct ButtonPressed {
///     pub id: String,
/// }
///
/// // In UI system: send event
/// fn ui_system(mut events: ResMut<Events<ButtonPressed>>) {
///     events.send(ButtonPressed { id: "play".into() });
/// }
///
/// // In game system: read events
/// fn game_system(mut events: ResMut<Events<ButtonPressed>>) {
///     for e in events.read() {
///         println!("Button pressed: {}", e.id);
///     }
///     events.clear();
/// }
/// ```
pub struct Events<T: 'static + Send + Sync> {
    events: VecDeque<T>,
    /// Track how many events were read in the current frame
    read_count: usize,
    _marker: PhantomData<T>,
}

impl<T: 'static + Send + Sync> Events<T> {
    /// Creates a new empty event queue.
    pub fn new() -> Self {
        Self {
            events: VecDeque::new(),
            read_count: 0,
            _marker: PhantomData,
        }
    }

    /// Sends an event into the queue.
    pub fn send(&mut self, event: T) {
        self.events.push_back(event);
    }

    /// Returns an iterator over all events in the queue.
    /// This does NOT consume the events - they remain in the queue
    /// until `clear()` is called.
    pub fn read(&self) -> impl Iterator<Item = &T> {
        self.events.iter()
    }

    /// Returns the number of events currently in the queue.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Returns true if there are no events in the queue.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Clears all events from the queue.
    /// This should be called after all systems have processed the events.
    pub fn clear(&mut self) {
        self.events.clear();
        self.read_count = 0;
    }

    /// Drains all events from the queue, returning them.
    /// This is useful when you want to consume events rather than just read them.
    pub fn drain(&mut self) -> impl Iterator<Item = T> + '_ {
        self.read_count = 0;
        self.events.drain(..)
    }

    /// Returns a reference to the most recent event, if any.
    pub fn last(&self) -> Option<&T> {
        self.events.back()
    }

    /// Returns true if there are unread events.
    /// Useful for systems that want to check if there's new work.
    pub fn has_pending(&self) -> bool {
        !self.events.is_empty()
    }
}

impl<T: 'static + Send + Sync> Default for Events<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: 'static + Send + Sync + std::fmt::Debug> std::fmt::Debug for Events<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Events")
            .field("count", &self.events.len())
            .field("events", &self.events)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct TestEvent {
        value: i32,
    }

    #[test]
    fn test_send_and_read() {
        let mut events = Events::<TestEvent>::new();
        events.send(TestEvent { value: 1 });
        events.send(TestEvent { value: 2 });

        let collected: Vec<&TestEvent> = events.read().collect();
        assert_eq!(collected.len(), 2);
        assert_eq!(collected[0].value, 1);
        assert_eq!(collected[1].value, 2);
    }

    #[test]
    fn test_clear() {
        let mut events = Events::<TestEvent>::new();
        events.send(TestEvent { value: 1 });
        events.send(TestEvent { value: 2 });

        assert_eq!(events.len(), 2);
        events.clear();
        assert_eq!(events.len(), 0);
        assert!(events.is_empty());
    }

    #[test]
    fn test_drain() {
        let mut events = Events::<TestEvent>::new();
        events.send(TestEvent { value: 1 });
        events.send(TestEvent { value: 2 });

        let drained: Vec<TestEvent> = events.drain().collect();
        assert_eq!(drained.len(), 2);
        assert_eq!(drained[0].value, 1);
        assert_eq!(drained[1].value, 2);
        assert!(events.is_empty());
    }

    #[test]
    fn test_last() {
        let mut events = Events::<TestEvent>::new();
        assert!(events.last().is_none());

        events.send(TestEvent { value: 1 });
        assert_eq!(events.last().unwrap().value, 1);

        events.send(TestEvent { value: 2 });
        assert_eq!(events.last().unwrap().value, 2);
    }

    #[test]
    fn test_has_pending() {
        let mut events = Events::<TestEvent>::new();
        assert!(!events.has_pending());

        events.send(TestEvent { value: 1 });
        assert!(events.has_pending());

        events.clear();
        assert!(!events.has_pending());
    }

    #[test]
    fn test_read_does_not_consume() {
        let mut events = Events::<TestEvent>::new();
        events.send(TestEvent { value: 42 });

        // First read
        let first: Vec<&TestEvent> = events.read().collect();
        assert_eq!(first.len(), 1);

        // Second read should still see the event
        let second: Vec<&TestEvent> = events.read().collect();
        assert_eq!(second.len(), 1);

        // Only after clear is it gone
        events.clear();
        let third: Vec<&TestEvent> = events.read().collect();
        assert_eq!(third.len(), 0);
    }

    #[test]
    fn test_default() {
        let events = Events::<TestEvent>::default();
        assert!(events.is_empty());
    }
}

use crate::Event;

pub type BoxedEvent = Box<dyn Event + Send + Sync>;

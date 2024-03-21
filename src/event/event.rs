pub trait Event {
    fn identifier(&self) -> String;
}

pub type BoxedEvent = Box<dyn Event + Send + Sync>;

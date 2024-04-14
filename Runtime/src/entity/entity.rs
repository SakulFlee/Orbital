use std::any::Any;

use ulid::Ulid;

pub trait Entity {
    fn new() -> Self
    where
        Self: Sized;

    fn ulid(&self) -> &Ulid;

    fn set_ulid(&mut self, ulid: Ulid);

    fn event_received(&mut self, identifier: String, event: &dyn Any);

    fn as_any(&self) -> &dyn Any;
}

pub type BoxedEntity = Box<dyn Entity + Send + Sync>;

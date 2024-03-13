use std::any::Any;

use ulid::Ulid;

pub trait Entity {
    fn ulid(&self) -> &Ulid;
    fn set_ulid(&mut self, ulid: Ulid);
    fn event_received(&mut self, identifier: String, event: &dyn Any);
    fn as_any(&self) -> &dyn Any;
}

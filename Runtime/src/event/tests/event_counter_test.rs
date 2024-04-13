use std::any::Any;

use ulid::Ulid;

use crate::{entity::Entity, event::Event};

pub struct EventTestCounter;

impl Event for EventTestCounter {
    fn identifier(&self) -> String {
        String::from("counter.test")
    }
}

pub struct EntityCountTest {
    ulid: Ulid,
    count: u8,
}

impl EntityCountTest {
    pub fn counter(&self) -> u8 {
        self.count
    }
}

impl Entity for EntityCountTest {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            ulid: Ulid::new(),
            count: 0,
        }
    }

    fn ulid(&self) -> &Ulid {
        &self.ulid
    }

    fn set_ulid(&mut self, ulid: Ulid) {
        self.ulid = ulid;
    }

    fn event_received(&mut self, _identifier: String, _event: &dyn Any) {
        self.count += 1;
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

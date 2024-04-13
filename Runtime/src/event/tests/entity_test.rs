use std::any::Any;

use ulid::Ulid;

use crate::entity::Entity;

pub struct EntityTest {
    ulid: Ulid,
    success: bool,
}

impl EntityTest {
    pub fn success(&self) -> bool {
        self.success
    }
}

impl Entity for EntityTest {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            ulid: Ulid::new(),
            success: false,
        }
    }

    fn ulid(&self) -> &Ulid {
        &self.ulid
    }

    fn set_ulid(&mut self, ulid: Ulid) {
        self.ulid = ulid;
    }

    fn event_received(&mut self, _identifier: String, _event: &dyn Any) {
        self.success = true;
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

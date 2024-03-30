use ulid::Ulid;

use crate::entity::Entity;

pub struct EntityTest {
    ulid: Ulid,
}

impl EntityTest {
    pub fn new() -> Self {
        Self { ulid: Ulid::new() }
    }
}

impl Entity for EntityTest {
    fn ulid(&self) -> &Ulid {
        &self.ulid
    }

    fn set_ulid(&mut self, ulid: Ulid) {
        self.ulid = ulid;
    }

    fn event_received(&mut self, _identifier: String, _event: &dyn std::any::Any) {
        unimplemented!()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

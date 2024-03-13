use ulid::Ulid;

pub trait Entity {
    fn ulid(&self) -> &Ulid;
    fn set_ulid(&mut self, ulid: Ulid);
}

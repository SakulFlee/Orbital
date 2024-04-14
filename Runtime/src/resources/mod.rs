use ulid::Ulid;

pub mod vertex;
pub use vertex::*;

pub mod mesh;
pub use mesh::*;

pub trait Resource {
    fn ulid(&self) -> &Ulid;

    fn set_ulid(&mut self, ulid: Ulid);
}

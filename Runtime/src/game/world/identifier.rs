use ulid::Ulid;

pub enum Identifier {
    Ulid(Ulid),
    Tag(String),
}

use ulid::Ulid;

#[derive(Debug)]
pub enum Identifier {
    Ulid(Ulid),
    Tag(String),
}

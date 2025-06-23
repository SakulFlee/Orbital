#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Origin {
    App,
    Element { label: String },
}

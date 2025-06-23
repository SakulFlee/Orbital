#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Origin {
    App,
    System,
    Element { label: String },
}

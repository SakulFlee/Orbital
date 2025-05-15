#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Target {
    App,
    Element { label: String },
}

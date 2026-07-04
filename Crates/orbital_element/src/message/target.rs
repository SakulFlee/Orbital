#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Target {
    Broadcast,
    Element {
        labels: Vec<String>,
    },
}

use crate::element::Element;

#[derive(Debug)]
pub enum ElementChange {
    Spawn(Box<dyn Element + Send + Sync>),
    Despawn(String),
    // TODO: Make singular
    AddLabels {
        element_label: String,
        new_labels: Vec<String>,
    },
    // TODO: Make singular
    RemoveLabels {
        element_label: String,
        new_labels: Vec<String>,
    },
}

use crate::element::{Element, Message};

#[derive(Debug)]
pub enum ElementEvent {
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
        labels_to_be_removed: Vec<String>,
    },
    SendMessage(Message),
}

use message::Message;

#[derive(Debug)]
pub enum ElementChange {
    Spawn(Box<dyn Element + Send + Sync>),
    Despawn(String),
    AddLabels {
        element_label: String,
        new_labels: Vec<String>,
    },
    RemoveLabels {
        element_label: String,
        new_labels: Vec<String>,
    },
}

use super::Event;

/// Used when registering an [Element](super::Element).
#[derive(Debug)]
pub struct ElementRegistration {
    /// Each [Element] must have at least one _label_ defined.
    /// Any additional _labels_ will work the same as the main _label_.
    /// [Element]s can share _labels_ to
    labels: Vec<String>,
    initial_world_changes: Vec<Event>,
}

impl ElementRegistration {
    pub fn new<S: Into<String>>(main_label: S) -> Self {
        Self {
            labels: vec![main_label.into()],
            initial_world_changes: Vec::new(),
        }
    }

    pub fn with_additional_label<S: Into<String>>(mut self, label: S) -> Self {
        self.labels.push(label.into());

        self
    }

    pub fn with_additional_labels<S: Into<String>>(mut self, labels: Vec<S>) -> Self {
        let processed_labels: Vec<String> = labels.into_iter().map(|s| s.into()).collect();
        self.labels.extend(processed_labels);

        self
    }

    pub fn with_initial_world_change(mut self, world_change: Event) -> Self {
        self.initial_world_changes.push(world_change);

        self
    }

    pub fn with_initial_world_changes(mut self, world_changes: Vec<Event>) -> Self {
        self.initial_world_changes.extend(world_changes);

        self
    }

    pub fn extract(self) -> (Vec<String>, Vec<Event>) {
        (self.labels, self.initial_world_changes)
    }
}

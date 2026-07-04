use crate::Event;

#[derive(Debug)]
pub struct ElementRegistration {
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

    pub fn with_initial_event(mut self, event: Event) -> Self {
        self.initial_world_changes.push(event);
        self
    }

    pub fn with_initial_events(mut self, events: Vec<Event>) -> Self {
        self.initial_world_changes.extend(events);
        self
    }

    pub fn extract(self) -> (Vec<String>, Vec<Event>) {
        (self.labels, self.initial_world_changes)
    }
}

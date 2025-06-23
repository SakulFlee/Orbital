use std::time::Instant;

use futures::{stream::FuturesUnordered, StreamExt};
use hashbrown::HashMap;
use log::{error, warn};

use super::{ElementEvent, Event};
use crate::{
    app::input::InputState,
    element::{Element, Message, Origin},
};

type ElementIndexType = u64;

#[derive(Debug)]
pub struct ElementStore
where
    Self: Send + Sync,
{
    element_map: HashMap<ElementIndexType, Box<dyn Element + Send + Sync>>,
    cursor_index: ElementIndexType,
    label_map: HashMap<String, ElementIndexType>,
    message_queue: HashMap<String, Vec<Message>>,
}

impl Default for ElementStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ElementStore {
    pub const MAX_TIME_IN_SECONDS: u64 = 5;

    pub fn new() -> Self {
        Self {
            element_map: HashMap::new(),
            cursor_index: ElementIndexType::MIN,
            label_map: HashMap::new(),
            message_queue: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.element_map.clear();
        self.cursor_index = 0;
        self.label_map.clear();
        self.message_queue.clear();
    }

    pub fn store_element(&mut self, element: Box<dyn Element + Send + Sync>, labels: Vec<String>) {
        let next_cursor_index = self.cursor_index + 1;
        self.cursor_index = next_cursor_index;
        self.element_map.insert(next_cursor_index, element);

        // Reserve capacity for better performance with large label vectors
        self.label_map.reserve(labels.len());
        for label in labels {
            self.label_map.insert(label, next_cursor_index);
        }
    }

    pub fn remove_element(&mut self, element_label: &str) {
        if let Some(element_id) = self.label_map.get(element_label).cloned() {
            self.element_map.remove(&element_id);

            self.label_map.retain(|_, v| element_id.eq(v));
        }
    }

    pub fn queue_message(&mut self, message: Message) {
        let label = match message.to() {
            Origin::Element { label } => label.clone(),
            Origin::App => {
                error!("Attempted queueing message in ElementStore with target 'App'. This is not allowed!");
                return;
            }
        };

        self.message_queue.entry(label).or_default().push(message);
    }

    pub async fn process_events(&mut self, events: Vec<ElementEvent>) -> Vec<Event> {
        let mut result_events = Vec::new();

        for event in events {
            match event {
                ElementEvent::Spawn(element) => {
                    let registration = element.on_registration();
                    let (labels, new_events) = registration.extract();

                    self.store_element(element, labels);

                    result_events.extend(new_events);
                }
                ElementEvent::Despawn(label) => self.remove_element(&label),
                ElementEvent::AddLabels {
                    element_label,
                    new_labels,
                } => self.add_label(&element_label, new_labels),
                ElementEvent::RemoveLabels {
                    element_label,
                    labels_to_be_removed,
                } => self.remove_label(&element_label, labels_to_be_removed),
                ElementEvent::SendMessage(message) => self.queue_message(message),
            }
        }

        result_events
    }
    pub async fn update(&mut self, delta_time: f64, input_state: &InputState) -> Vec<Event> {
        // Draining here will remove the messages from the queue so we don't need to clean/clear after!
        let mut messages = self.message_queue.drain().collect::<HashMap<_, _>>();

        let x = self
            .element_map
            .iter_mut()
            // Loop over each element and retrieve its label
            .map(|(element_id, element)| {
                (
                    self.label_map
                        .iter()
                        .filter_map(|(label, id)| if element_id == id { Some(label) } else { None })
                        .collect::<Vec<_>>(),
                    element,
                )
            })
            // For each element label, retrieve messages from the queue
            .map(|(label, element)| {
                let messages = label
                    .into_iter()
                    .filter_map(|label| messages.remove(label))
                    .flatten()
                    .collect::<Vec<_>>();

                let messages_option = if messages.is_empty() {
                    None
                } else {
                    Some(messages)
                };

                // Call the update function or without messages
                element.on_update(delta_time, input_state, messages_option)
            })
            // Await all futures
            .collect::<FuturesUnordered<_>>()
            .filter_map(|changes| async move { changes })
            .flat_map(futures::stream::iter)
            .collect()
            .await;

        if !messages.is_empty() {
            let now = Instant::now();
            messages
                .values_mut()
                .for_each(|messages| {
                    messages.retain(|message| {
                        if message.creation_instant().duration_since(now).as_secs() >= Self::MAX_TIME_IN_SECONDS {
                            warn!("Message has exceeded the maximum time of {} seconds. Removing message: '{:?}'", Self::MAX_TIME_IN_SECONDS, message);
                            return false;
                        }

                        true
                    });
                });

            self.message_queue.extend(messages);
        }

        x
    }

    pub fn add_label(&mut self, element_label: &str, new_labels: Vec<String>) {
        if let Some(element_id) = self.label_map.get(element_label).cloned() {
            new_labels.into_iter().for_each(|label| {
                self.label_map.insert(label, element_id);
            })
        }
    }

    pub fn remove_label(&mut self, element_label: &str, labels_to_be_removed: Vec<String>) {
        if let Some(element_id) = self.label_map.get(element_label).cloned() {
            self.label_map
                .retain(|k, v| element_id.eq(v) && labels_to_be_removed.contains(k));
        }
    }

    pub fn label_to_index(&self, label: &str) -> Option<ElementIndexType> {
        self.label_map.get(label).cloned()
    }

    pub fn element_count(&self) -> usize {
        self.element_map.len()
    }
}

use std::time::Instant;

use futures::{stream::FuturesUnordered, StreamExt};
use hashbrown::{hash_map::Entry, HashMap};
use log::warn;

use crate::input::InputState;

use super::{Element, Message, WorldChange};

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
        // Get the next cursor index.
        // Realistically, this should never overflow ...
        let next_cursor_index =   self.cursor_index.checked_add(1).unwrap_or_else(|| panic!("Congratulations! You managed to run out of Element indices! This means, you have spawned {} Elements already and are attempting to spawn another one. Here's a question from me to you: How do you have so much memory?", ElementIndexType::MAX));

        // Update the cursor position to the current new index
        self.cursor_index = next_cursor_index;

        // Insert the Element
        self.element_map.insert(next_cursor_index, element);

        // For each label, add the label as a key and index as the value
        labels.into_iter().for_each(|label| {
            self.label_map.insert(label, next_cursor_index);
        });
    }

    pub fn remove_element(&mut self, element_label: &str) {
        if let Some(element_id) = self.label_map.get(element_label).cloned() {
            self.element_map.remove(&element_id);

            self.label_map.retain(|_, v| element_id.eq(v));
        }
    }

    pub fn queue_message(&mut self, message: Message) {
        // TODO: Potentially unnecessary string conversions!
        match self.message_queue.entry(message.to().to_string()) {
            Entry::Occupied(mut occupied_entry) => {
                occupied_entry.get_mut().push(message);
            }
            Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(vec![message]);
            }
        }
    }

    pub async fn update(&mut self, delta_time: f64, input_state: &InputState) -> Vec<WorldChange> {
        // Draining here will remove the messages from the queue so we don't need to clean/clear after!
        let mut messages = self.message_queue.drain().collect::<HashMap<_, _>>();

        let x = self
            .element_map
            .iter_mut()
            .map(|(element_id, element)| {
                (
                    self.label_map
                        .iter()
                        .find(|(_, id)| element_id == *id)
                        .map(|(label, _)| label)
                        .expect("Cannot find element label from id! This should never happen..."),
                    element,
                )
            })
            .map(|(label, element)| {
                let messages = messages.remove(label);
                element.on_update(delta_time, input_state, messages)
            })
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

    pub fn element_count(&self) -> usize {
        self.element_map.len()
    }
}

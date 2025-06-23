use std::error::Error;
use std::future::Future;
use std::sync::Arc;
use std::time::Instant;

use super::{ElementEvent, Event, Target};
use crate::{
    app::input::InputState,
    element::{Element, Message, Origin},
};
use futures::future::{join_all, JoinAll};
use futures::{stream::FuturesUnordered, StreamExt};
use hashbrown::HashMap;
use log::{error, warn};
use winit::event_loop::DeviceEvents;

type ElementIndexType = u64;

#[derive(Debug)]
pub struct ElementStore
where
    Self: Send + Sync,
{
    element_map: HashMap<ElementIndexType, Box<dyn Element + Send + Sync>>,
    cursor_index: ElementIndexType,
    label_map: HashMap<String, ElementIndexType>,
    message_queue: HashMap<ElementIndexType, Vec<Arc<Message>>>,
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
        self.message_queue.insert(next_cursor_index, Vec::new());

        // Reserve capacity for better performance with large label vectors
        self.label_map.reserve(labels.len());
        for label in labels {
            self.label_map.insert(label, next_cursor_index);
        }
    }

    pub fn remove_element(&mut self, element_label: &str) {
        if let Some(element_id) = self.label_map.get(element_label).cloned() {
            self.element_map.remove(&element_id);
            self.message_queue.remove(&element_id);

            self.label_map.retain(|_, v| element_id.eq(v));
        }
    }

    pub fn queue_message(&mut self, message: Message) {
        let labels = match message.to() {
            Target::Broadcast => self.label_map.keys().cloned().collect(),
            Target::Element { labels } => labels.to_owned(),
        };

        let arc = Arc::new(message);
        for label in labels {
            let idx = match self.label_to_index(&label) {
                None => {
                    warn!("Trying to queue message {arc:#?} but couldn't find element with label '{label}'!");
                    continue;
                }
                Some(label) => label,
            };

            let messages = self
                .message_queue
                .get_mut(&idx)
                .expect("Element must have a message queue!");
            messages.push(arc.clone());
        }
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

    async fn send_messages(&mut self) -> Vec<Event> {
        let messages = std::mem::take(&mut self.message_queue);
        let mut events = Vec::new();

        for (element_id, messages) in messages {
            match self.element_map.get_mut(&element_id) {
                None => {
                    warn!("Got a message in queue that is supposed to be send to element with ID #{element_id}, but element does not exist! Messages to be dropped: {messages:#?}");
                    continue;
                }
                Some(element) => {
                    for message in messages {
                        if let Some(new_events) = element.on_message(&message).await {
                            events.extend(new_events);
                        }
                    }
                }
            }
        }

        events
    }

    pub async fn update(&mut self, delta_time: f64, input_state: &InputState) -> Vec<Event> {
        let mut events = self.send_messages().await;

        let futures: Vec<_> = self
            .element_map
            .iter_mut()
            .map(|(_, x)| x.on_update(delta_time, input_state))
            .collect();

        let future_results = join_all(futures).await;
        let new_events: Vec<Event> = future_results.into_iter().flatten().flatten().collect();
        events.extend(new_events);

        events
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

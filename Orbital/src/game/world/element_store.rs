use hashbrown::HashMap;

use crate::{error::Error, input::InputState, variant::Variant};

use super::{Element, WorldChange};

type ElementIndexType = u64;

#[derive(Debug)]
pub struct ElementStore {
    pub element_map: HashMap<ElementIndexType, Box<dyn Element + Send>>,
    pub cursor_index: ElementIndexType,
    pub label_map: HashMap<String, ElementIndexType>,
}

impl ElementStore {
    pub fn new() -> Self {
        Self {
            element_map: HashMap::new(),
            cursor_index: 0,
            label_map: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.element_map.clear();
        self.cursor_index = 0;
        self.label_map.clear();
    }

    pub fn store_element(&mut self, element: Box<dyn Element>, labels: Vec<String>) {
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

    pub fn send_messages(
        &mut self,
        element_label: &str,
        messages: Vec<HashMap<String, Variant>>,
    ) -> Result<Vec<WorldChange>, Error> {
        if let Some(element_id) = self.label_map.get(element_label).cloned() {
            let element = self
                .element_map
                .get_mut(&element_id)
                .expect("Element label found and ID resolved, but Element doesn't exist.");

            let world_changes: Vec<_> = messages
                .into_iter()
                .filter_map(|message| element.on_message(message))
                .flatten()
                .collect();

            Ok(world_changes)
        } else {
            Err(Error::NotFound)
        }
    }

    pub fn send_focus_change(&mut self, focused: bool) {
        self.element_map
            .values_mut()
            .for_each(|x| x.on_focus_change(focused));
    }

    pub fn update(&mut self, delta_time: f64, input_state: &InputState) -> Vec<WorldChange> {
        self.element_map
            .values_mut()
            .filter_map(|x| x.on_update(delta_time, input_state))
            .flatten()
            .collect::<Vec<_>>()
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

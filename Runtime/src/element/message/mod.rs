use hashbrown::HashMap;
use std::time::Instant;

mod target;
pub use target::*;

mod variant;
pub use variant::*;

#[derive(Debug)]
pub struct Message {
    from: String,
    to: Origin,
    creation_instant: Instant,
    content: HashMap<String, Variant>,
}

impl Message {
    pub fn new_from_message(from: String, to: Origin, content: HashMap<String, Variant>) -> Self {
        Self {
            from,
            to,
            creation_instant: Instant::now(),
            content,
        }
    }

    pub fn new(from: String, to: Origin) -> Self {
        Self {
            from,
            to,
            creation_instant: Instant::now(),
            content: HashMap::new(),
        }
    }

    pub fn add_content(&mut self, key: String, value: Variant) {
        self.content.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<&Variant> {
        self.content.get(key)
    }

    pub fn from(&self) -> &str {
        &self.from
    }

    pub fn to(&self) -> &Origin {
        &self.to
    }

    pub fn creation_instant(&self) -> &Instant {
        &self.creation_instant
    }

    pub fn content(&self) -> &HashMap<String, Variant> {
        &self.content
    }
}

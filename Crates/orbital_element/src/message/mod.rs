use hashbrown::HashMap;
use std::time::Instant;

mod origin;
pub use origin::*;

mod target;
pub use target::*;

mod variant;
pub use variant::*;

#[derive(Debug)]
pub struct Message {
    from: Origin,
    to: Target,
    creation_instant: Instant,
    content: HashMap<String, Variant>,
}

impl Message {
    pub fn new(from: Origin, to: Target) -> Self {
        Self {
            from,
            to,
            creation_instant: Instant::now(),
            content: HashMap::new(),
        }
    }

    pub fn add_content(mut self, key: String, value: Variant) -> Self {
        self.content.insert(key, value);
        self
    }

    pub fn get(&self, key: &str) -> Option<&Variant> {
        self.content.get(key)
    }

    pub fn from(&self) -> &Origin {
        &self.from
    }

    pub fn to(&self) -> &Target {
        &self.to
    }

    pub fn creation_instant(&self) -> &Instant {
        &self.creation_instant
    }

    pub fn content(&self) -> &HashMap<String, Variant> {
        &self.content
    }
}

use super::{update_frequency, UpdateFrequency};

pub struct EntityConfiguration {
    tag: String,
    update_frequency: UpdateFrequency,
    render: bool,
}

impl EntityConfiguration {
    pub fn new<S>(tag: S, update_frequency: UpdateFrequency, render: bool) -> Self
    where
        S: Into<String>,
    {
        Self {
            tag: tag.into(),
            update_frequency,
            render,
        }
    }

    pub fn get_tag(&self) -> &str {
        self.tag.as_ref()
    }

    pub fn get_update_frequency(&self) -> &UpdateFrequency {
        &self.update_frequency
    }

    pub fn get_render(&self) -> bool {
        self.render
    }
}

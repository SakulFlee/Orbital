use super::UpdateFrequency;

pub struct EntityConfiguration {
    tag: String,
    update_frequency: UpdateFrequency,
    do_render: bool,
}

impl EntityConfiguration {
    pub fn new<S>(tag: S, update_frequency: UpdateFrequency, do_render: bool) -> Self
    where
        S: Into<String>,
    {
        Self {
            tag: tag.into(),
            update_frequency,
            do_render,
        }
    }

    pub fn tag(&self) -> &str {
        self.tag.as_ref()
    }

    pub fn update_frequency(&self) -> &UpdateFrequency {
        &self.update_frequency
    }

    pub fn do_render(&self) -> bool {
        self.do_render
    }
}

use crate::app::impl_component;

#[derive(Debug, Default)]
pub struct Health {
    health: u32,
}

impl Health {
    pub fn new(health: u32) -> Self {
        Self { health }
    }

    pub fn get_health(&self) -> u32 {
        self.health
    }

    pub fn set_health(&mut self, health: u32) {
        self.health = health;
    }
}

impl_component!(Health);

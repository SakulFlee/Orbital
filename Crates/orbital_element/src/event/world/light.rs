use orbital_resources::LightDescriptor;

#[derive(Debug)]
pub enum LightEvent {
    Spawn(LightDescriptor),
    Despawn(String),
    Update(String, LightDescriptor),
}

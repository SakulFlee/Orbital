use orbital::{
    game::{Element, ElementRegistration},
    resources::descriptors::{ImportDescriptor, InstanceDescriptor, Instancing, ModelDescriptor},
    ulid::Ulid,
};

pub struct ChessCube;

impl Element for ChessCube {
    fn on_registration(&mut self, _ulid: &Ulid) -> ElementRegistration {
        // We can directly supply our ModelDescriptor during registration.
        // Alternatively, we could queue a WorldChange::SpawnModel(Owned).
        ElementRegistration {
            models: Some(vec![ModelDescriptor::FromGLTF(
                "Assets/Models/DamagedHelmet.glb",
                ImportDescriptor::Index(0),
                ImportDescriptor::Index(0),
                Instancing::Single(InstanceDescriptor::default()),
            )]),
            ..Default::default()
        }
    }
}

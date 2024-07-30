use orbital::{
    game::{Element, ElementRegistration, Identifier, WorldChange},
    hashbrown::HashMap,
    input::InputFrame,
    log::info,
    ulid::Ulid,
    variant::Variant,
};

#[derive(Debug, Default)]
pub struct TestElement {
    own_id: Option<Ulid>,
}

impl Element for TestElement {
    fn on_registration(&mut self, ulid: &Ulid) -> ElementRegistration {
        self.own_id = Some(*ulid);

        ElementRegistration::default()
    }

    fn on_update(
        &mut self,
        _delta_time: f64,
        _input_frame: &Option<&InputFrame>,
    ) -> Option<Vec<WorldChange>> {
        // Where this message should be send to. In this case ourself.
        let target_id = Identifier::Ulid(self.own_id.unwrap());
        // The message itself, just a simple String
        let mut message = HashMap::new();
        message.insert("msg".into(), Variant::String("Hello, World!".into()));
        // Queue the message for sending
        Some(vec![WorldChange::SendMessage(target_id, message)])
    }

    fn on_message(&mut self, message: HashMap<String, Variant>) -> Option<Vec<WorldChange>> {
        info!("On Message: {:#?}", message);

        None
    }
}

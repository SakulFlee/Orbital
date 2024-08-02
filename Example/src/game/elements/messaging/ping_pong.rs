use orbital::{
    game::{Element, ElementRegistration, Identifier, WorldChange},
    hashbrown::HashMap,
    log::{info, warn},
    ulid::Ulid,
    variant::Variant,
};

pub struct PingPongElement {
    is_ping: bool,
}

impl PingPongElement {
    pub fn new(is_ping: bool) -> Self {
        Self { is_ping }
    }
}

impl Element for PingPongElement {
    fn on_registration(&mut self, _ulid: &Ulid) -> ElementRegistration {
        if self.is_ping {
            let mut message = HashMap::new();
            message.insert("payload".into(), Variant::Boolean(true));

            let world_change = vec![WorldChange::SendMessage(
                Identifier::Tag("Pong".into()),
                message,
            )];

            ElementRegistration {
                tags: Some(vec!["Ping".into()]),
                world_changes: Some(world_change),
                ..Default::default()
            }
        } else {
            ElementRegistration {
                tags: Some(vec!["Pong".into()]),
                ..Default::default()
            }
        }
    }

    fn on_message(&mut self, message: HashMap<String, Variant>) -> Option<Vec<WorldChange>> {
        if let Some(Variant::Boolean(is_ping)) = message.get("payload") {
            if self.is_ping && !*is_ping {
                // info!("Pong received! Sending Ping back.");

                let mut message = HashMap::new();
                message.insert("payload".into(), Variant::Boolean(self.is_ping));

                return Some(vec![WorldChange::SendMessage(
                    Identifier::Tag("Pong".into()),
                    message,
                )]);
            } else if !self.is_ping && *is_ping {
                // info!("Ping received! Sending Pong back.");

                let mut message = HashMap::new();
                message.insert("payload".into(), Variant::Boolean(self.is_ping));

                return Some(vec![WorldChange::SendMessage(
                    Identifier::Tag("Ping".into()),
                    message,
                )]);
            } else {
                warn!(
                    "Neither Ping nor Pong packet match! Self: {}; Packet: {}",
                    self.is_ping, is_ping
                );
            }
        }

        None
    }
}

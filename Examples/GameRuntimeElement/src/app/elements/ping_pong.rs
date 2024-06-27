use akimo_runtime::{
    game::{Element, ElementRegistration, Identifier, WorldChange},
    hashbrown::HashMap,
    log::{info, warn},
    ulid::Ulid,
    variant::Variant,
};

pub struct PingPongElement {
    is_ping: bool,
    changes_queue: Vec<WorldChange>,
}

impl PingPongElement {
    pub fn new(is_ping: bool) -> Self {
        let mut initial_changes = Vec::new();
        if is_ping {
            let mut message = HashMap::<String, Variant>::new();
            message.insert("payload".into(), Variant::Boolean(true));

            initial_changes.push(WorldChange::SendMessage(
                Identifier::Tag("Pong".into()),
                message,
            ));
        }

        Self {
            is_ping,
            changes_queue: initial_changes,
        }
    }
}

impl Element for PingPongElement {
    fn on_registration(&mut self, _ulid: &Ulid) -> ElementRegistration {
        ElementRegistration {
            tags: Some(vec![if self.is_ping {
                "Ping".into()
            } else {
                "Pong".into()
            }]),
            ..Default::default()
        }
    }

    fn on_update(&mut self, _delta_time: f64) -> Option<Vec<WorldChange>> {
        Some(self.changes_queue.drain(..).collect())
    }

    fn on_message(&mut self, message: HashMap<String, Variant>) {
        info!("On Message: {:#?}", message);

        if let Some(Variant::Boolean(is_ping)) = message.get("payload") {
            if self.is_ping && !*is_ping {
                info!("Pong received! Sending Ping back.");

                let mut message = HashMap::new();
                message.insert("payload".into(), Variant::Boolean(self.is_ping));

                self.changes_queue.push(WorldChange::SendMessage(
                    Identifier::Tag("Pong".into()),
                    message,
                ));
            } else if !self.is_ping && *is_ping {
                info!("Ping received! Sending Pong back.");

                let mut message = HashMap::new();
                message.insert("payload".into(), Variant::Boolean(self.is_ping));

                self.changes_queue.push(WorldChange::SendMessage(
                    Identifier::Tag("Ping".into()),
                    message,
                ));
            } else {
                warn!(
                    "Neither Ping nor Pong packet match! Self: {}; Packet: {}",
                    self.is_ping, is_ping
                );
            }
        }
    }
}

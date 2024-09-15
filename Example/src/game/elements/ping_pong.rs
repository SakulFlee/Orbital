use orbital::{
    game::{Element, ElementRegistration, WorldChange},
    hashbrown::HashMap,
    log::warn,
    variant::Variant,
};

#[derive(Debug)]
pub struct PingPongElement {
    is_ping: bool,
}

impl PingPongElement {
    pub fn new(is_ping: bool) -> Self {
        Self { is_ping }
    }
}

impl Element for PingPongElement {
    fn on_registration(&mut self) -> ElementRegistration {
        let element_registration = ElementRegistration::new("PingPong");

        if self.is_ping {
            let mut message = HashMap::new();
            message.insert("payload".into(), Variant::Boolean(true));

            element_registration
                .with_additional_label("Ping")
                .with_initial_world_change(WorldChange::SendMessage("Pong".to_string(), message))
        } else {
            element_registration.with_additional_label("Pong")
        }
    }

    fn on_message(&mut self, message: HashMap<String, Variant>) -> Option<Vec<WorldChange>> {
        if let Some(Variant::Boolean(is_ping)) = message.get("payload") {
            if self.is_ping && !*is_ping {
                // info!("Pong received! Sending Ping back.");

                let mut message = HashMap::new();
                message.insert("payload".into(), Variant::Boolean(self.is_ping));

                return Some(vec![WorldChange::SendMessage("Pong".into(), message)]);
            } else if !self.is_ping && *is_ping {
                // info!("Ping received! Sending Pong back.");

                let mut message = HashMap::new();
                message.insert("payload".into(), Variant::Boolean(self.is_ping));

                return Some(vec![WorldChange::SendMessage("Ping".into(), message)]);
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

use orbital::{
    async_trait::async_trait,
    element::{Element, ElementEvent, ElementRegistration, Event, Message, Target, Variant},
    app::input::InputState,
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

impl PingPongElement {
    const LABEL_PING: &'static str = "Ping";
    const LABEL_PONG: &'static str = "Pong";
}

#[async_trait]
impl Element for PingPongElement {
    fn on_registration(&self) -> ElementRegistration {
        let element_registration = ElementRegistration::new("PingPong");

        if self.is_ping {
            let mut message = Message::new(
                Self::LABEL_PING.to_string(),
                Target::Element { label: Self::LABEL_PONG.to_string() }
            );
            message.add_content("payload".to_string(), Variant::Boolean(self.is_ping));

            element_registration
                .with_additional_label(Self::LABEL_PING)
                .with_initial_world_change(Event::Element(ElementEvent::SendMessage(message)))
        } else {
            element_registration.with_additional_label(Self::LABEL_PONG)
        }
    }

    async fn on_update(
        &mut self,
        _delta_time: f64,
        _input_state: &InputState,
        messages_option: Option<Vec<Message>>,
    ) -> Option<Vec<Event>> {
        if let Some(messages) = messages_option {
            for message in messages {
                if let Some(payload) = message.get("payload") {
                    if let Variant::Boolean(is_ping) = payload {
                        if *is_ping && !self.is_ping {
                            // Disabled to prevent log spam!
                            // debug!("Ping Received! Sending Pong back.");
                            let mut message = Message::new(
                                Self::LABEL_PONG.to_string(),
                                Target::Element { label: Self::LABEL_PING.to_string() }
                            );
                            message.add_content("payload".to_string(), Variant::Boolean(self.is_ping));

                            return Some(vec![Event::Element(ElementEvent::SendMessage(message))]);
                        } else if !*is_ping && self.is_ping {
                            // Disabled to prevent log spam!
                            // debug!("Pong Received! Sending Ping back.");
                            let mut message = Message::new(
                                Self::LABEL_PING.to_string(),
                                Target::Element { label: Self::LABEL_PONG.to_string() }
                            );
                            message.add_content("payload".to_string(), Variant::Boolean(self.is_ping));

                            return Some(vec![Event::Element(ElementEvent::SendMessage(message))]);
                        } else {
                            panic!(
                                "Received Ping payload as Ping element, or, Pong payload as Pong element. This should never happen!"
                            );
                        }
                    }
                }
            }
        }

        None
    }
}

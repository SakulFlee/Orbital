use orbital::logging::{debug, error, info, warn};
use orbital::{
    app::input::InputState,
    async_trait::async_trait,
    element::{Element, ElementEvent, ElementRegistration, Event, Message, Target, Variant},
};

#[derive(Debug)]
pub struct PingPongElement {
    is_ping: bool,
}

impl PingPongElement {
    const LABEL_PING: &'static str = "Ping";
    const LABEL_PONG: &'static str = "Pong";

    pub fn new(is_ping: bool) -> Self {
        Self { is_ping }
    }

    fn make_message(&self) -> Message {
        let mut message = Message::new(
            if self.is_ping {
                Self::LABEL_PING.to_string()
            } else {
                Self::LABEL_PONG.to_string()
            },
            Target::Element {
                label: if self.is_ping {
                    Self::LABEL_PONG.to_string()
                } else {
                    Self::LABEL_PING.to_string()
                },
            },
        );

        message.add_content("payload".to_string(), Variant::Boolean(self.is_ping));

        message
    }
}

#[async_trait]
impl Element for PingPongElement {
    fn on_registration(&self) -> ElementRegistration {
        let element_registration = ElementRegistration::new("PingPong");

        if self.is_ping {
            let message = self.make_message();
            debug!("Initial message: {:?}", message);

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
            if messages.len() > 1 {
                warn!(
                    "Received more than one message in one update cycle. This should never happen! Only the first message will be processed."
                );
            }

            let message = &messages[0];
            let payload = match message.get("payload") {
                Some(x) => x,
                None => {
                    error!("Received message without payload. This should never happen!");
                    return None;
                }
            };

            return if let Variant::Boolean(is_ping) = payload {
                let valid = *is_ping != self.is_ping;
                if !valid {
                    error!(
                        "Expected different payloads, but received the same payload. This should never happen!"
                    );
                    return None;
                }
                info!(
                    "Received valid payload! Is ping? Self: {}; Payload: {}",
                    self.is_ping, is_ping
                );

                let message = self.make_message();
                Some(vec![Event::Element(ElementEvent::SendMessage(message))])
            } else {
                error!("Received message with invalid payload. This should never happen!");
                None
            };
        }

        None
    }
}

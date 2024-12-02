use orbital::{
    async_trait::async_trait,
    world::{Element, ElementRegistration, Message, WorldChange},
    hashbrown::HashMap,
    input::InputState,
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

impl PingPongElement {
    const LABEL_PING: &'static str = "Ping";
    const LABEL_PONG: &'static str = "Pong";
}

#[async_trait]
impl Element for PingPongElement {
    fn on_registration(&mut self) -> ElementRegistration {
        let element_registration = ElementRegistration::new("PingPong");

        if self.is_ping {
            element_registration
                .with_additional_label(Self::LABEL_PING)
                .with_initial_world_change(WorldChange::SendMessage(Message::new_from_message(
                    Self::LABEL_PING.into(),
                    Self::LABEL_PONG.into(),
                    HashMap::from_iter(vec![("payload".into(), Variant::Boolean(self.is_ping))]),
                )))
        } else {
            element_registration.with_additional_label(Self::LABEL_PONG)
        }
    }

    async fn on_update(
        &mut self,
        _delta_time: f64,
        _input_state: &InputState,
        messages_option: Option<Vec<Message>>,
    ) -> Option<Vec<WorldChange>> {
        if let Some(messages) = messages_option {
            for message in messages {
                if let Some(payload) = message.get("payload") {
                    if let Variant::Boolean(is_ping) = payload {
                        if *is_ping && !self.is_ping {
                            // Disabled to prevent log spam!
                            // debug!("Ping Received! Sending Pong back.");
                            return Some(vec![WorldChange::SendMessage(
                                Message::new_from_message(
                                    Self::LABEL_PONG.into(),
                                    Self::LABEL_PING.into(),
                                    HashMap::from_iter(vec![(
                                        "payload".into(),
                                        Variant::Boolean(self.is_ping),
                                    )]),
                                ),
                            )]);
                        } else if !*is_ping && self.is_ping {
                            // Disabled to prevent log spam!
                            // debug!("Pong Received! Sending Ping back.");
                            return Some(vec![WorldChange::SendMessage(
                                Message::new_from_message(
                                    Self::LABEL_PING.into(),
                                    Self::LABEL_PONG.into(),
                                    HashMap::from_iter(vec![(
                                        "payload".into(),
                                        Variant::Boolean(self.is_ping),
                                    )]),
                                ),
                            )]);
                        } else {
                            panic!("Received Ping payload as Ping element, or, Pong payload as Pong element. This should never happen!");
                        }
                    }
                }
            }
        }

        // None

        // match messages_option {
        //     Some(messages) => {
        //         panic!("{:?}", messages);
        //     }
        //     None => (),
        // }

        // if let Some(messages) = &messages_option {
        //     debug!("HIT: {:?}", messages);
        // }

        // if messages_option.is_none() {
        //     return None;
        // }

        // if let Some(messages) = messages_option {
        //     debug!("Received messages: {:?}", messages);
        //     for message in messages {
        //         if let Some(Variant::Boolean(is_ping)) = message.get("payload") {
        //             debug!("Self: {}, Received: {}", self.is_ping, is_ping);
        //             if self.is_ping && !*is_ping {
        //                 info!("Pong received! Sending Ping back.");

        //                 let mut message = Message::new("Ping".into(), "Pong".into());
        //                 message
        //                     .add_content(String::from("payload"), Variant::Boolean(self.is_ping));

        //                 return Some(vec![WorldChange::SendMessage(message)]);
        //             } else if !self.is_ping && *is_ping {
        //                 info!("Ping received! Sending Pong back.");

        //                 let mut message = Message::new("Pong".into(), "Ping".into());
        //                 message
        //                     .add_content(String::from("payload"), Variant::Boolean(self.is_ping));

        //                 return Some(vec![WorldChange::SendMessage(message)]);
        //             } else {
        //                 panic!(
        //                     "Neither Ping nor Pong packet match! Self: {}; Packet: {}",
        //                     self.is_ping, is_ping
        //                 );
        //             }
        //         }
        //     }
        // }

        None
    }
}

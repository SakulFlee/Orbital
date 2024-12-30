use orbital::{
    async_trait::async_trait,
    input::{InputButton, InputState},
    log::debug,
    variant::Variant,
    winit::keyboard::{KeyCode, PhysicalKey},
    world::{Element, ElementRegistration, Message, WorldChange},
};

#[derive(Debug, Default)]
pub struct DebugController {
    delta: f64,
    debug_wireframes: bool,
}

impl DebugController {
    pub const IDENTIFIER: &'static str = "DEBUG";
    pub const RENDERER_IDENTIFIER: &'static str = "RENDERER";

    pub const TRIGGER_KEY: InputButton =
        InputButton::Keyboard(PhysicalKey::Code(KeyCode::ControlLeft));
    pub const DEBUG_WIREFRAMES_KEY: InputButton =
        InputButton::Keyboard(PhysicalKey::Code(KeyCode::Digit4));

    pub const KEY_DEBUG_WIREFRAMES_ENABLED: &'static str = "debug_wireframes_enabled";
}

#[async_trait]
impl Element for DebugController {
    fn on_registration(&self) -> ElementRegistration {
        ElementRegistration::new(Self::IDENTIFIER)
    }

    async fn on_update(
        &mut self,
        delta_time: f64,
        input_state: &InputState,
        _messages: Option<Vec<Message>>,
    ) -> Option<Vec<WorldChange>> {
        let mut world_changes = Vec::new();

        self.delta += delta_time;
        if self.delta < 1.0 {
            return None;
        }
        self.delta -= 1.0;

        let inputs =
            input_state.button_state_many(&[&Self::TRIGGER_KEY, &Self::DEBUG_WIREFRAMES_KEY]);

        if let Some((_, triggered)) = inputs.get(&Self::TRIGGER_KEY) {
            if !triggered {
                return None;
            }

            if let Some((_, triggered)) = inputs.get(&Self::DEBUG_WIREFRAMES_KEY) {
                if !triggered {
                    return None;
                }

                if !self.debug_wireframes {
                    self.debug_wireframes = true;

                    let mut message = Message::new(
                        Self::IDENTIFIER.to_string(),
                        Self::RENDERER_IDENTIFIER.to_string(),
                    );
                    message.add_content(
                        Self::KEY_DEBUG_WIREFRAMES_ENABLED.to_string(),
                        Variant::Boolean(self.debug_wireframes),
                    );

                    world_changes.push(WorldChange::SendMessageToApp(message));
                } else {
                    if self.debug_wireframes {
                        self.debug_wireframes = false;

                        let mut message = Message::new(
                            Self::IDENTIFIER.to_string(),
                            Self::RENDERER_IDENTIFIER.to_string(),
                        );
                        message.add_content(
                            Self::KEY_DEBUG_WIREFRAMES_ENABLED.to_string(),
                            Variant::Boolean(self.debug_wireframes),
                        );

                        world_changes.push(WorldChange::SendMessageToApp(message));
                    }
                }
            }
        }

        if !world_changes.is_empty() {
            Some(world_changes)
        } else {
            None
        }
    }
}

use orbital::{
    async_trait::async_trait,
    input::{InputButton, InputState},
    variant::Variant,
    winit::keyboard::{KeyCode, PhysicalKey},
    world_old::{Element, ElementRegistration, Message, WorldChange},
};

#[derive(Debug, Default)]
pub struct DebugController {
    timeout_delta: Option<f64>,
}

impl DebugController {
    pub const IDENTIFIER: &'static str = "DEBUG";
    pub const RENDERER_IDENTIFIER: &'static str = "RENDERER";

    pub const TRIGGER_KEY: InputButton =
        InputButton::Keyboard(PhysicalKey::Code(KeyCode::ControlLeft));
    pub const DEBUG_WIREFRAMES_KEY: InputButton =
        InputButton::Keyboard(PhysicalKey::Code(KeyCode::Digit4));
    pub const DEBUG_BOUNDING_BOX_WIREFRAMES_KEY: InputButton =
        InputButton::Keyboard(PhysicalKey::Code(KeyCode::Digit5));
    pub const DEBUG_FREEZE_FRUSTUM_KEY: InputButton =
        InputButton::Keyboard(PhysicalKey::Code(KeyCode::Digit6));

    pub const KEY_DEBUG_WIREFRAMES_ENABLED: &'static str = "debug_wireframes_enabled";
    pub const KEY_DEBUG_BOUNDING_BOX_WIREFRAMES_ENABLED: &'static str =
        "debug_bounding_box_wireframe_enabled";
    pub const KEY_DEBUG_FREEZE_FRUSTUM: &'static str = "debug_freeze_frustum";

    pub const INPUT_TIMEOUT: f64 = 0.4;
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

        if let Some(ref mut timeout) = self.timeout_delta {
            *timeout -= delta_time;
            if *timeout < 0.0 {
                self.timeout_delta = None;
            }
        }

        // Skip if we are still in timeout
        if self.timeout_delta.is_some() {
            return None;
        }

        let inputs = input_state.button_state_many(&[
            &Self::TRIGGER_KEY,
            &Self::DEBUG_WIREFRAMES_KEY,
            &Self::DEBUG_BOUNDING_BOX_WIREFRAMES_KEY,
            &Self::DEBUG_FREEZE_FRUSTUM_KEY,
        ]);

        if let Some((_, triggered)) = inputs.get(&Self::TRIGGER_KEY) {
            if !triggered {
                return None;
            }

            if let Some((_, triggered)) = inputs.get(&Self::DEBUG_WIREFRAMES_KEY) {
                if *triggered {
                    self.timeout_delta = Some(Self::INPUT_TIMEOUT);

                    let mut message = Message::new(
                        Self::IDENTIFIER.to_string(),
                        Self::RENDERER_IDENTIFIER.to_string(),
                    );
                    message.add_content(
                        Self::KEY_DEBUG_WIREFRAMES_ENABLED.to_string(),
                        Variant::Empty,
                    );
                    world_changes.push(WorldChange::SendMessageToApp(message));
                }
            }

            if let Some((_, triggered)) = inputs.get(&Self::DEBUG_BOUNDING_BOX_WIREFRAMES_KEY) {
                if *triggered {
                    self.timeout_delta = Some(Self::INPUT_TIMEOUT);

                    let mut message = Message::new(
                        Self::IDENTIFIER.to_string(),
                        Self::RENDERER_IDENTIFIER.to_string(),
                    );
                    message.add_content(
                        Self::KEY_DEBUG_BOUNDING_BOX_WIREFRAMES_ENABLED.to_string(),
                        Variant::Empty,
                    );
                    world_changes.push(WorldChange::SendMessageToApp(message));
                }
            }

            if let Some((_, triggered)) = inputs.get(&Self::DEBUG_FREEZE_FRUSTUM_KEY) {
                if *triggered {
                    self.timeout_delta = Some(Self::INPUT_TIMEOUT);

                    let mut message = Message::new(
                        Self::IDENTIFIER.to_string(),
                        Self::RENDERER_IDENTIFIER.to_string(),
                    );
                    message.add_content(Self::KEY_DEBUG_FREEZE_FRUSTUM.to_string(), Variant::Empty);
                    world_changes.push(WorldChange::SendMessageToApp(message));
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

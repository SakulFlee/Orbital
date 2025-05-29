use std::time::Instant;

use orbital::{
    async_trait::async_trait,
    input::{InputButton, InputState},
    log::debug,
    resources::descriptors::SkyboxType,
    winit::keyboard::{KeyCode, PhysicalKey},
    world_old::{Element, ElementRegistration, Message, WorldChange},
};

#[derive(Debug)]
pub struct DebugWorldEnvironment {
    current_skybox_type: SkyboxType,
    last_trigger: Instant,
}

impl Default for DebugWorldEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

impl DebugWorldEnvironment {
    pub const KEY_DEBUG_DOWN: InputButton =
        InputButton::Keyboard(PhysicalKey::Code(KeyCode::Digit1));
    pub const KEY_DEBUG_RESET: InputButton =
        InputButton::Keyboard(PhysicalKey::Code(KeyCode::Digit2));
    pub const KEY_DEBUG_UP: InputButton = InputButton::Keyboard(PhysicalKey::Code(KeyCode::Digit3));

    pub fn new() -> Self {
        Self {
            current_skybox_type: SkyboxType::default(),
            last_trigger: Instant::now(),
        }
    }
}

#[async_trait]
impl Element for DebugWorldEnvironment {
    fn on_registration(&self) -> ElementRegistration {
        ElementRegistration::new("debug_world_environment")
    }

    async fn on_update(
        &mut self,
        _delta_time: f64,
        input_state: &InputState,
        _messages: Option<Vec<Message>>,
    ) -> Option<Vec<WorldChange>> {
        if self.last_trigger.elapsed().as_secs() < 1 {
            return None;
        }

        if let Some(pressed) = input_state
            .button_state_any(&Self::KEY_DEBUG_UP)
            .map(|(_, pressed)| pressed)
        {
            if pressed {
                match self.current_skybox_type {
                    SkyboxType::Diffuse => {
                        self.current_skybox_type = SkyboxType::Specular { lod: 0 };
                    }
                    SkyboxType::Specular { lod } => {
                        if lod == 10 {
                            self.current_skybox_type = SkyboxType::Diffuse;
                        } else {
                            self.current_skybox_type = SkyboxType::Specular { lod: lod + 1 };
                        }
                    }
                }
                self.last_trigger = Instant::now();

                debug!("Changing skybox to {:?}!", self.current_skybox_type);
                return Some(vec![WorldChange::ChangeWorldEnvironmentSkyboxType {
                    skybox_type: self.current_skybox_type,
                }]);
            }
        }

        if let Some(pressed) = input_state
            .button_state_any(&Self::KEY_DEBUG_DOWN)
            .map(|(_, pressed)| pressed)
        {
            if pressed {
                match self.current_skybox_type {
                    SkyboxType::Diffuse => {
                        self.current_skybox_type = SkyboxType::Specular { lod: 10 };
                    }
                    SkyboxType::Specular { lod } => {
                        if lod == 0 {
                            self.current_skybox_type = SkyboxType::Diffuse;
                        } else {
                            self.current_skybox_type = SkyboxType::Specular { lod: lod - 1 };
                        }
                    }
                }
                self.last_trigger = Instant::now();

                debug!("Changing skybox to {:?}!", self.current_skybox_type);
                return Some(vec![WorldChange::ChangeWorldEnvironmentSkyboxType {
                    skybox_type: self.current_skybox_type,
                }]);
            }
        }

        if let Some(pressed) = input_state
            .button_state_any(&Self::KEY_DEBUG_RESET)
            .map(|(_, pressed)| pressed)
        {
            if pressed {
                self.current_skybox_type = SkyboxType::Specular { lod: 0 };
                debug!("Resetting skybox to {:?}!", self.current_skybox_type);

                self.last_trigger = Instant::now();
                return Some(vec![WorldChange::ChangeWorldEnvironmentSkyboxType {
                    skybox_type: self.current_skybox_type,
                }]);
            }
        }

        None
    }
}

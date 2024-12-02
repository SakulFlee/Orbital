use std::time::Instant;

use orbital::{
    async_trait::async_trait,
    world::{Element, ElementRegistration, Message, WorldChange},
    input::InputState,
    log::debug,
    resources::descriptors::SkyboxType,
    util::InputHandler,
    winit::keyboard::{KeyCode, PhysicalKey},
};

#[derive(Debug)]
pub struct DebugWorldEnvironment {
    input_handler: InputHandler,
    current_skybox_type: SkyboxType,
    last_trigger: Instant,
}

impl Default for DebugWorldEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

impl DebugWorldEnvironment {
    pub const KEY_DEBUG_UP: PhysicalKey = PhysicalKey::Code(KeyCode::Digit2);
    pub const KEY_DEBUG_DOWN: PhysicalKey = PhysicalKey::Code(KeyCode::Digit1);

    pub const ACTION_DEBUG_UP: &'static str = "debug_up";
    pub const ACTION_DEBUG_DOWN: &'static str = "debug_down";

    pub fn new() -> Self {
        let mut input_handler = InputHandler::new();

        input_handler.register_keyboard_mapping(Self::KEY_DEBUG_UP, Self::ACTION_DEBUG_UP);
        input_handler.register_keyboard_mapping(Self::KEY_DEBUG_DOWN, Self::ACTION_DEBUG_DOWN);

        Self {
            current_skybox_type: SkyboxType::default(),
            input_handler,
            last_trigger: Instant::now(),
        }
    }
}

#[async_trait]
impl Element for DebugWorldEnvironment {
    fn on_registration(&mut self) -> ElementRegistration {
        ElementRegistration::new("debug_world_environment")
    }
    
    async fn on_update(
        &mut self,
        _delta_time: f64,
        _input_state: &InputState,
        _messages: Option<Vec<Message>>,
    ) -> Option<Vec<WorldChange>> {
        if self.last_trigger.elapsed().as_secs() < 1 {
            return None;
        }

        if self.input_handler.is_triggered(Self::ACTION_DEBUG_UP) {
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
            Some(vec![WorldChange::ChangeWorldEnvironmentSkyboxType {
                skybox_type: self.current_skybox_type,
            }])
        } else if self.input_handler.is_triggered(Self::ACTION_DEBUG_DOWN) {
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
            Some(vec![WorldChange::ChangeWorldEnvironmentSkyboxType {
                skybox_type: self.current_skybox_type,
            }])
        } else {
            None
        }
    }
}

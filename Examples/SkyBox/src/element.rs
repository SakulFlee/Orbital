use std::time::Instant;

use orbital::element::EnvironmentEvent;
use orbital::resources::WorldEnvironmentDescriptor;
use orbital::{
    app::input::{InputButton, InputState},
    async_trait::async_trait,
    element::{Element, ElementRegistration, Event, WorldEvent},
    winit::keyboard::{KeyCode, PhysicalKey},
};

#[derive(Debug)]
pub struct DebugWorldEnvironment {
    current_world_environment: WorldEnvironments,
    last_trigger: Instant,
}

impl Default for DebugWorldEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Default)]
enum WorldEnvironments {
    PhotoStudio,
    #[default]
    Kloppenheim,
    LonelyRoad,
}

impl WorldEnvironments {
    fn to_descriptor(&self) -> WorldEnvironmentDescriptor {
        WorldEnvironmentDescriptor::FromFile {
            cube_face_size: WorldEnvironmentDescriptor::DEFAULT_SIZE,
            path: self.to_path(),
            sampling_type: WorldEnvironmentDescriptor::DEFAULT_SAMPLING_TYPE,
            custom_specular_mip_level_count: None,
        }
    }

    fn to_path(&self) -> String {
        match self {
            WorldEnvironments::PhotoStudio => {
                "Assets/WorldEnvironments/PhotoStudio.hdr".to_owned()
            }
            WorldEnvironments::Kloppenheim => {
                "Assets/WorldEnvironments/Kloppenheim.hdr".to_owned()
            }
            WorldEnvironments::LonelyRoad => {
                "Assets/WorldEnvironments/LonelyRoad.hdr".to_owned()
            }
        }
    }

    fn next(&self) -> Self {
        match self {
            WorldEnvironments::PhotoStudio => WorldEnvironments::LonelyRoad,
            WorldEnvironments::Kloppenheim => WorldEnvironments::PhotoStudio,
            WorldEnvironments::LonelyRoad => WorldEnvironments::Kloppenheim,
        }
    }
}

impl DebugWorldEnvironment {
    pub const KEY_DEBUG_NEXT: InputButton =
        InputButton::Keyboard(PhysicalKey::Code(KeyCode::Space));

    pub fn new() -> Self {
        Self {
            current_world_environment: WorldEnvironments::default(),
            last_trigger: Instant::now(),
        }
    }
}

#[async_trait]
impl Element for DebugWorldEnvironment {
    fn on_registration(&self) -> ElementRegistration {
        ElementRegistration::new("debug_world_environment").with_initial_world_change(Event::World(
            WorldEvent::Environment(EnvironmentEvent::Change {
                descriptor: self.current_world_environment.to_descriptor(),
                enable_ibl: true,
            }),
        ))
    }

    async fn on_update(
        &mut self,
        _delta_time: f64,
        input_state: &InputState,
    ) -> Option<Vec<Event>> {
        if self.last_trigger.elapsed().as_secs() < 1 {
            return None;
        }

        if let Some(pressed) = input_state
            .button_state_any(&Self::KEY_DEBUG_NEXT)
            .map(|(_, pressed)| pressed)
        {
            if !pressed {
                return None;
            }

            self.last_trigger = Instant::now();

            self.current_world_environment = self.current_world_environment.next();
            let descriptor = self.current_world_environment.to_descriptor();
            return Some(vec![Event::World(WorldEvent::Environment(
                EnvironmentEvent::Change {
                    descriptor,
                    enable_ibl: true,
                },
            ))]);
        }

        None
    }
}

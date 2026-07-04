use crate::ButtonAxis;
use cgmath::Vector3;
use orbital_input::{InputAxis, InputButton};

#[derive(Debug, Clone, PartialEq)]
pub enum CameraControllerMovementType {
    Input {
        axis: Option<InputAxis>,
        button_axis: Option<Vec<ButtonAxis>>,
        button_up: Option<InputButton>,
        button_down: Option<InputButton>,
        speed: f32,
        ignore_pitch_for_forward_movement: bool,
        axis_dead_zone: f64,
    },
    Following {
        label: String,
        offset: Vector3<f32>,
        rotate_around_target: bool,
        follow_target_entity_rotation: bool,
    },
    Static,
}

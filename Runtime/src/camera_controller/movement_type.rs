use crate::app::input::{InputAxis, InputButton};
use crate::camera_controller::ButtonAxis;
use cgmath::Vector3;

#[derive(Debug, Clone, PartialEq)]
pub enum CameraControllerMovementType {
    /// Directly listens for inputs and moves the camera accordingly.
    Input {
        /// Axis for movement
        axis: Option<InputAxis>,
        /// Button axis for movement
        button_axis: Option<Vec<ButtonAxis>>,
        /// Button to move up
        button_up: Option<InputButton>,
        /// Button to move down
        button_down: Option<InputButton>,
        /// Speed that the camera moves at.
        speed: f32,
        /// If true, the camera will ignore the pitch when moving forward.
        /// Meaning, only the yaw value will be used to determine where "forward" is.
        /// If false, the camera will take pitch into consideration.
        ///
        /// In most cases you want this set to true so that the forward vector doesn't move the
        /// camera up and down.
        /// However, there are some exceptions like, for example, _"creative flight"_, _free flight_,
        /// diving/swimming or space.
        /// TODO: Check if it's actually pitch and not yaw
        ignore_pitch_for_forward_movement: bool,
    },
    /// Follows an entity with a given offset.
    Following {
        /// The entity label to follow.
        label: String,
        /// The offset from the entity to the camera.
        /// This is often referred to as a "spring arm".
        ///
        /// This offset is in local space.
        /// The forward vector will be determined (see [`CameraControllerRotationType`]) and offset based on:
        /// X+ = forward
        /// X- = backward
        /// Y+ = right
        /// Y- = left
        /// Z+ = up
        /// Z- = down
        /// TODO: Double check
        offset: Vector3<f32>,
        /// If true, the camera will rotate around the follower target entity, instead of the camera itself.
        /// In the 3rd-person view you most likely want this set to `true` to revolve around the 3rd-person character.
        /// In 1st person view, you most likely want this set to `false` to directly rotate the camera from its position.
        rotate_around_target: bool,
        /// If true, the camera will rotate with the target entity.
        /// If false, the camera will stay in its current position, regardless of target entity rotation.
        follow_target_entity_rotation: bool,
    },
    /// The camera is not moving automatically and will stay in the same position.
    /// The camera can still be interacted with and manually change positions!
    Static,
}

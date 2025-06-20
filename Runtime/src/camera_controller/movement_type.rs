use crate::app::input::{InputAxis, InputButton};
use crate::or::Or;
use cgmath::Vector3;

#[derive(Debug, Clone, PartialEq)]
pub enum CameraControllerMovementType {
    /// Directly listens for inputs and moves the camera accordingly.
    Input {
        /// Buttons to move forward.
        move_forward: Option<Vec<Or<InputButton, InputAxis>>>,
        /// Buttons to move backward.
        move_backward: Option<Vec<Or<InputButton, InputAxis>>>,
        /// Buttons to move left.
        move_left: Option<Vec<Or<InputButton, InputAxis>>>,
        /// Buttons to move right.
        move_right: Option<Vec<Or<InputButton, InputAxis>>>,
        /// Buttons to move up.
        move_up: Option<Vec<Or<InputButton, InputAxis>>>,
        /// Buttons to move down.
        move_down: Option<Vec<Or<InputButton, InputAxis>>>,
        /// Speed that the camera moves at.
        speed: f32,
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

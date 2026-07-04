use orbital_input::InputAxis;

#[derive(Debug, Clone, PartialEq)]
pub struct CameraControllerAxisInputMode {
    pub axis: Vec<InputAxis>,
    pub sensitivity: f32,
}

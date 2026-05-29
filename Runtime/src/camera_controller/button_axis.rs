use crate::app::input::InputButton;

#[derive(Debug, Clone, PartialEq)]
pub struct ButtonAxis {
    pub forward: InputButton,
    pub backward: InputButton,
    pub left: InputButton,
    pub right: InputButton,
}

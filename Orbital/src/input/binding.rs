use winit::event::{AxisId, ButtonId, MouseButton};

#[derive(Debug, Hash, PartialEq, PartialOrd, Eq, Ord, Clone, Copy)]
pub enum InputBinding {
    KeyboardKey { key: u32 },
    MouseButton { button: MouseButton },
    MouseWheel { is_x: bool, is_increasing: bool },
    GamepadButton { button: ButtonId },
    GamepadAxis { axis_id: AxisId, increasing: bool },
}

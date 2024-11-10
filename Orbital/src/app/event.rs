use winit::event::{DeviceEvent, DeviceId, StartCause, WindowEvent};

#[derive(Debug, PartialEq, Clone)]
pub enum AppEvent {
    WindowEvent(WindowEvent),
    NewEventCycle(StartCause),
    DeviceEvent(DeviceId, DeviceEvent),
    MemoryWarning,
    Suspended,
    Resumed,
    #[cfg(feature = "gamepad_input")]
    GamepadInput(gilrs::Event),
}

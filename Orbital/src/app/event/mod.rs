use winit::event::{DeviceEvent, DeviceId, StartCause, WindowEvent};

mod input;
pub use input::*;

#[derive(Debug, PartialEq)]
pub enum AppEvent {
    Resumed,
    Suspended,
    InputEvent(InputEvent),
    WindowEvent(WindowEvent),
    NewEventCycle(StartCause),
    DeviceEvent(DeviceId, DeviceEvent),
    MemoryWarning,
}

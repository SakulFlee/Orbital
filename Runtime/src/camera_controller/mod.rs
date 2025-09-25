//! # Camera Controller Module
//!
//! The camera controller module provides a flexible system for controlling cameras
//! in the Orbital engine. It supports various movement and rotation types, input
//! methods, and camera behaviors.
//!
//! ## Key Components
//!
//! - **Descriptor**: Defines camera controller configuration
//! - **MovementType**: Different ways the camera can move (e.g., free, orbital, first-person)
//! - **RotationType**: Different ways the camera can rotate (e.g., mouse look, controller)
//! - **Input Types**: Various input methods including mouse, keyboard, and gamepad
//! - **Realization**: Runtime representation of the camera controller
//!
//! ## Usage
//!
//! Camera controllers are configured through descriptors and then realized at runtime
//! to control camera movement and rotation based on user input.

mod descriptor;
pub use descriptor::*;

mod movement_type;
pub use movement_type::*;

mod button_axis;
pub use button_axis::*;

mod rotation_type;
pub use rotation_type::*;

mod mouse_input;
pub use mouse_input::*;

mod mouse_input_type;
pub use mouse_input_type::*;

mod axis_input;
pub use axis_input::*;

mod button_input;
pub use button_input::*;

mod realization;
pub use realization::*;

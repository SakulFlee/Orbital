//! # Texture Module
//!
//! Generic, reusable GPU texture and buffer resource types. These are not
//! tied to the renderer or shader systems — they can be used by any crate
//! that needs to describe or manage GPU textures and buffers.

mod buffer;
pub use buffer::*;

mod texture;
pub use texture::*;

mod size;
pub use size::*;

mod error;
pub use error::*;

mod descriptor;
pub use descriptor::*;

mod filter_mode;
pub use filter_mode::*;

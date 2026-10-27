//! Camera-facing textured billboards.
//!
//! Billboard components are realized into ordinary models, so they reuse the
//! renderer's opaque/deferred path, forward transparency queue, depth testing,
//! shadow filtering and sorted transparent draws.

mod component;
pub use component::{Billboard, BillboardMode};

mod material;

mod system;
pub use system::BillboardModule;

mod text;
pub use text::{text_billboard, text_billboard_with_background};

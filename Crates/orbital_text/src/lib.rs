//! # Orbital Text
//!
//! Text rendering for the Orbital engine using SDF (Signed Distance Fields).
//!
//! Provides:
//! - [`FontData`] — font loading and glyph metrics
//! - [`SdfAtlas`] — texture atlas for SDF glyphs
//! - Text mesh generation for both 2D screen-space and 3D billboard rendering
//!
//! ## Quick Start
//!
//! ```ignore
//! use orbital_text::{FontData, TextConfig, generate_text_mesh};
//!
//! // Load a font
//! let font_data = include_bytes!("path/to/font.ttf");
//! let mut font = FontData::from_bytes(font_data).unwrap();
//!
//! // Generate text mesh
//! let config = TextConfig {
//!     font_size: 24.0,
//!     color: [1.0, 1.0, 1.0, 1.0],
//!     ..Default::default()
//! };
//! let vertices = generate_text_mesh("Hello, World!", &mut font, &config);
//! ```

pub mod atlas;
pub mod font;
pub mod text_mesh;

pub use atlas::{SdfAtlas, ShelfPacker};
pub use font::{FontData, GlyphInfo};
pub use text_mesh::{generate_billboard_text_mesh, generate_text_mesh, measure_text, TextConfig};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn font_loading() {
        let font_data = include_bytes!("../assets/default.ttf");
        let font = FontData::from_bytes(font_data);
        assert!(font.is_ok());
    }

    #[test]
    fn text_rendering_pipeline() {
        let font_data = include_bytes!("../assets/default.ttf");
        let mut font = FontData::from_bytes(font_data).unwrap();

        let config = TextConfig {
            font_size: 32.0,
            color: [1.0, 0.0, 0.0, 1.0],
            ..Default::default()
        };

        let vertices = generate_text_mesh("Test", &mut font, &config);
        assert!(!vertices.is_empty());

        let (width, height) = measure_text("Test", &font, 32.0);
        assert!(width > 0.0);
        assert!(height > 0.0);
    }
}

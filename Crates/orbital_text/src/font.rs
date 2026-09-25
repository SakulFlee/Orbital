use std::collections::HashMap;

/// Metadata for a single glyph in the font.
#[derive(Debug, Clone, Copy)]
pub struct GlyphInfo {
    /// UV coordinates (min corner) in the atlas texture.
    pub uv_min: [f32; 2],
    /// UV coordinates (max corner) in the atlas texture.
    pub uv_max: [f32; 2],
    /// Glyph size in pixels.
    pub size: [f32; 2],
    /// Offset from the baseline to the top-left of the glyph.
    pub offset: [f32; 2],
    /// Horizontal advance to the next character.
    pub advance: f32,
}

/// A loaded font with glyph metrics.
pub struct FontData {
    /// The fontdue font instance.
    font: fontdue::Font,
    /// Cache of glyph metrics at different sizes.
    glyph_cache: HashMap<u32, HashMap<char, GlyphInfo>>,
}

impl FontData {
    /// Creates a new FontData from font bytes.
    pub fn from_bytes(data: &[u8]) -> Result<Self, &'static str> {
        let font = fontdue::Font::from_bytes(data, fontdue::FontSettings::default())
            .map_err(|_| "Failed to parse font")?;

        Ok(Self {
            font,
            glyph_cache: HashMap::new(),
        })
    }

    /// Creates a new FontData from a fontdue::Font.
    pub fn from_fontdue(font: fontdue::Font) -> Self {
        Self {
            font,
            glyph_cache: HashMap::new(),
        }
    }

    /// Returns a reference to the underlying fontdue font.
    pub fn font(&self) -> &fontdue::Font {
        &self.font
    }

    /// Gets or generates glyph metrics for a character at a given font size.
    ///
    /// This generates the SDF rasterization and computes metrics.
    /// The glyph data is cached for reuse.
    pub fn get_glyph(&mut self, ch: char, font_size: u32, sdf_scale: f32) -> &GlyphInfo {
        let size_cache = self.glyph_cache.entry(font_size).or_default();

        if let std::collections::hash_map::Entry::Vacant(e) = size_cache.entry(ch) {
            let (metrics, bitmap) = self.font.rasterize(ch, font_size as f32);

            // Generate SDF from the rasterized bitmap
            let _sdf_bitmap = generate_sdf(&bitmap, metrics.width, metrics.height, sdf_scale);

            // For now, store placeholder UV coordinates
            // The actual atlas packing happens in the atlas module
            let glyph = GlyphInfo {
                uv_min: [0.0, 0.0],
                uv_max: [1.0, 1.0],
                size: [metrics.width as f32, metrics.height as f32],
                offset: [metrics.xmin as f32, metrics.ymin as f32],
                advance: metrics.advance_width,
            };

            e.insert(glyph);
        }

        size_cache.get(&ch).unwrap()
    }

    /// Measures the width of a string at a given font size.
    pub fn measure_width(&self, text: &str, font_size: f32) -> f32 {
        let mut width = 0.0;
        for ch in text.chars() {
            let (metrics, _) = self.font.rasterize(ch, font_size);
            width += metrics.advance_width;
        }
        width
    }

    /// Returns the line height for a given font size.
    pub fn line_height(&self, font_size: f32) -> f32 {
        let (metrics, _) = self.font.rasterize('x', font_size);
        metrics.height as f32 + metrics.ymin as f32
    }
}

/// Generates a signed distance field from a rasterized bitmap.
///
/// The SDF represents the distance to the nearest edge, with:
/// - Negative values inside the glyph
/// - Positive values outside the glyph
/// - 0 at the edge
pub fn generate_sdf(bitmap: &[u8], width: usize, height: usize, scale: f32) -> Vec<f32> {
    if width == 0 || height == 0 {
        return Vec::new();
    }

    let spread = (scale * 8.0) as i32; // Distance in pixels to compute SDF
    let mut sdf = vec![0.0f32; width * height];

    for y in 0..height {
        for x in 0..width {
            let pixel = bitmap[y * width + x] as f32 / 255.0;

            // Find minimum distance to an edge
            let mut min_dist = f32::MAX;

            for dy in -spread..=spread {
                for dx in -spread..=spread {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;

                    if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                        let neighbor = bitmap[ny as usize * width + nx as usize] as f32 / 255.0;

                        // Edge detected if there's a significant difference
                        if (pixel - neighbor).abs() > 0.1 {
                            let dist = ((dx * dx + dy * dy) as f32).sqrt();
                            min_dist = min_dist.min(dist);
                        }
                    }
                }
            }

            // Convert to signed distance (standard convention: positive inside, negative outside)
            if pixel > 0.5 {
                // Inside the glyph → positive
                sdf[y * width + x] = min_dist / spread as f32;
            } else {
                // Outside the glyph → negative
                sdf[y * width + x] = -min_dist / spread as f32;
            }
        }
    }

    sdf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn font_data_creation() {
        // Use a built-in font for testing
        let font_data = include_bytes!("../../orbital_text/assets/default.ttf");
        let font = FontData::from_bytes(font_data);
        assert!(font.is_ok());
    }

    #[test]
    fn measure_width() {
        let font_data = include_bytes!("../../orbital_text/assets/default.ttf");
        let font = FontData::from_bytes(font_data).unwrap();

        let width = font.measure_width("Hello", 24.0);
        assert!(width > 0.0);
    }

    #[test]
    fn line_height() {
        let font_data = include_bytes!("../../orbital_text/assets/default.ttf");
        let font = FontData::from_bytes(font_data).unwrap();

        let height = font.line_height(24.0);
        assert!(height > 0.0);
    }
}

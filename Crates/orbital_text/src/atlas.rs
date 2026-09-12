use crate::font::{FontData, GlyphInfo};

/// A texture atlas containing SDF glyphs.
#[derive(Debug)]
pub struct SdfAtlas {
    /// Width of the atlas texture in pixels.
    pub width: u32,
    /// Height of the atlas texture in pixels.
    pub height: u32,
    /// Raw pixel data (RGBA).
    pub data: Vec<u8>,
    /// Glyph metrics indexed by character.
    pub glyphs: std::collections::HashMap<char, GlyphInfo>,
}

impl SdfAtlas {
    /// Creates a new empty atlas with the given dimensions.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            data: vec![0; (width * height * 4) as usize],
            glyphs: std::collections::HashMap::new(),
        }
    }

    /// Packs a glyph into the atlas at the given position.
    ///
    /// This is a simple shelf packing algorithm.
    pub fn pack_glyph(
        &mut self,
        ch: char,
        sdf_data: &[f32],
        width: usize,
        height: usize,
        offset_x: u32,
        offset_y: u32,
    ) -> GlyphInfo {
        let atlas_width = self.width as usize;
        let atlas_height = self.height as usize;

        // Copy SDF data into atlas (convert float to RGBA)
        for y in 0..height {
            for x in 0..width {
                let atlas_x = offset_x as usize + x;
                let atlas_y = offset_y as usize + y;

                if atlas_x < atlas_width && atlas_y < atlas_height {
                    let sdf_value = sdf_data[y * width + x];
                    // Map SDF from [-1, 1] to [0, 255]
                    let byte_value = ((sdf_value + 1.0) * 0.5 * 255.0).clamp(0.0, 255.0) as u8;

                    let idx = (atlas_y * atlas_width + atlas_x) * 4;
                    self.data[idx] = byte_value;     // R
                    self.data[idx + 1] = byte_value; // G
                    self.data[idx + 2] = byte_value; // B
                    self.data[idx + 3] = 255;        // A
                }
            }
        }

        let glyph = GlyphInfo {
            uv_min: [
                offset_x as f32 / self.width as f32,
                offset_y as f32 / self.height as f32,
            ],
            uv_max: [
                (offset_x + width as u32) as f32 / self.width as f32,
                (offset_y + height as u32) as f32 / self.height as f32,
            ],
            size: [width as f32, height as f32],
            offset: [0.0, 0.0],
            advance: width as f32,
        };

        self.glyphs.insert(ch, glyph);
        glyph
    }

    /// Returns the glyph info for a character, if it exists in the atlas.
    pub fn get_glyph(&self, ch: char) -> Option<&GlyphInfo> {
        self.glyphs.get(&ch)
    }

    /// Returns the raw RGBA pixel data.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Builds an SDF atlas from a font for the given character set.
    ///
    /// Generates SDF data for each character and packs it into the atlas.
    /// Returns the populated atlas with glyph UV coordinates.
    pub fn build_atlas(
        font: &mut FontData,
        font_size: u32,
        characters: &str,
        sdf_scale: f32,
        atlas_width: u32,
        atlas_height: u32,
    ) -> Self {
        let mut atlas = SdfAtlas::new(atlas_width, atlas_height);
        let mut packer = ShelfPacker::new(atlas_width, atlas_height);

        for ch in characters.chars() {
            // Get glyph info (generates SDF and caches it)
            let _glyph_info = font.get_glyph(ch, font_size, sdf_scale);

            // Get the SDF bitmap from fontdue
            let (metrics, bitmap) = font.font().rasterize(ch, font_size as f32);

            if metrics.width == 0 || metrics.height == 0 {
                continue;
            }

            // Generate SDF from the bitmap
            let sdf_data = crate::font::generate_sdf(
                &bitmap,
                metrics.width,
                metrics.height,
                sdf_scale,
            );

            // Allocate space in the atlas
            if let Some((offset_x, offset_y)) = packer.allocate(
                metrics.width as u32 + 2, // +2 for padding
                metrics.height as u32 + 2,
            ) {
                // Pack the glyph into the atlas
                atlas.pack_glyph(
                    ch,
                    &sdf_data,
                    metrics.width,
                    metrics.height,
                    offset_x + 1, // +1 for padding
                    offset_y + 1,
                );
            }
        }

        atlas
    }
}

/// Simple shelf packer for atlas generation.
pub struct ShelfPacker {
    current_x: u32,
    current_y: u32,
    row_height: u32,
    atlas_width: u32,
    atlas_height: u32,
}

impl ShelfPacker {
    pub fn new(atlas_width: u32, atlas_height: u32) -> Self {
        Self {
            current_x: 0,
            current_y: 0,
            row_height: 0,
            atlas_width,
            atlas_height,
        }
    }

    /// Attempts to allocate space for a glyph.
    /// Returns (x, y) if successful, or None if the atlas is full.
    pub fn allocate(&mut self, width: u32, height: u32) -> Option<(u32, u32)> {
        if self.current_x + width > self.atlas_width {
            // Move to next row
            self.current_x = 0;
            self.current_y += self.row_height;
            self.row_height = 0;
        }

        if self.current_y + height > self.atlas_height {
            return None; // Atlas is full
        }

        let x = self.current_x;
        let y = self.current_y;

        self.current_x += width;
        self.row_height = self.row_height.max(height);

        Some((x, y))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atlas_creation() {
        let atlas = SdfAtlas::new(512, 512);
        assert_eq!(atlas.width, 512);
        assert_eq!(atlas.height, 512);
        assert!(atlas.glyphs.is_empty());
    }

    #[test]
    fn shelf_packer_basic() {
        let mut packer = ShelfPacker::new(256, 256);

        let pos1 = packer.allocate(32, 32);
        assert_eq!(pos1, Some((0, 0)));

        let pos2 = packer.allocate(32, 32);
        assert_eq!(pos2, Some((32, 0)));

        let pos3 = packer.allocate(32, 32);
        assert_eq!(pos3, Some((64, 0)));
    }

    #[test]
    fn shelf_packer_row_wrap() {
        let mut packer = ShelfPacker::new(100, 100);

        // Fill first row
        let _ = packer.allocate(50, 20);
        let _ = packer.allocate(50, 20);

        // Next allocation should wrap to new row
        let pos = packer.allocate(30, 30);
        assert_eq!(pos, Some((0, 20)));
    }

    #[test]
    fn pack_glyph_into_atlas() {
        let mut atlas = SdfAtlas::new(256, 256);
        let sdf_data = vec![0.0; 16 * 16]; // 16x16 SDF

        let glyph = atlas.pack_glyph('A', &sdf_data, 16, 16, 0, 0);
        assert!(atlas.get_glyph('A').is_some());
        assert_eq!(glyph.size, [16.0, 16.0]);
    }
}

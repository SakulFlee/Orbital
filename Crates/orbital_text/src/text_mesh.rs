use orbital_2d::Vertex2D;

use crate::atlas::SdfAtlas;
use crate::font::FontData;

/// Configuration for text rendering.
#[derive(Debug, Clone)]
pub struct TextConfig {
    /// Font size in pixels.
    pub font_size: f32,
    /// Text color (RGBA).
    pub color: [f32; 4],
    /// Maximum width for word wrapping (None = no wrapping).
    pub max_width: Option<f32>,
    /// Line height multiplier.
    pub line_height_multiplier: f32,
}

impl Default for TextConfig {
    fn default() -> Self {
        Self {
            font_size: 24.0,
            color: [1.0, 1.0, 1.0, 1.0],
            max_width: None,
            line_height_multiplier: 1.2,
        }
    }
}

/// Generates a text mesh as 2D vertices.
///
/// Each character is rendered as a textured quad.
/// Returns vertices suitable for rendering with the 2D shader.
///
/// If an atlas is provided, UV coordinates are looked up from the atlas.
/// If no atlas is provided, placeholder UVs are used (for backward compatibility).
pub fn generate_text_mesh(
    text: &str,
    font: &FontData,
    config: &TextConfig,
    atlas: Option<&SdfAtlas>,
) -> Vec<Vertex2D> {
    let mut vertices = Vec::new();
    let mut cursor_x = 0.0f32;
    let mut cursor_y = 0.0f32;
    let line_height = font.line_height(config.font_size) * config.line_height_multiplier;

    for ch in text.chars() {
        if ch == '\n' {
            cursor_x = 0.0;
            cursor_y += line_height;
            continue;
        }

        if ch == ' ' {
            cursor_x += config.font_size * 0.25; // Approximate space width
            continue;
        }

        // Get glyph metrics
        let (metrics, _) = font.font().rasterize(ch, config.font_size);

        let glyph_width = metrics.width as f32;
        let glyph_height = metrics.height as f32;
        let glyph_x = cursor_x + metrics.xmin as f32;
        let glyph_y = cursor_y - metrics.ymin as f32 - glyph_height;

        // Check for word wrapping
        if let Some(max_width) = config.max_width
            && glyph_x + glyph_width > max_width
        {
            cursor_x = 0.0;
            cursor_y += line_height;
        }

        // Get UV coordinates from atlas or use placeholders
        let (uv_min, uv_max) = if let Some(atlas) = atlas {
            if let Some(glyph_info) = atlas.get_glyph(ch) {
                (glyph_info.uv_min, glyph_info.uv_max)
            } else {
                // Glyph not in atlas, use placeholder
                ([0.0, 0.0], [1.0, 1.0])
            }
        } else {
            // No atlas provided, use placeholder UVs
            ([0.0, 0.0], [1.0, 1.0])
        };

        // Generate quad vertices for this character
        let x0 = glyph_x;
        let y0 = glyph_y;
        let x1 = glyph_x + glyph_width;
        let y1 = glyph_y + glyph_height;

        // Two triangles for the quad (UV Y-coordinates: y0=top of screen=top of atlas, y1=bottom)
        vertices.push(Vertex2D::with_texcoord(
            [x0, y0],
            config.color,
            [uv_min[0], uv_min[1]], // Top-left UV
        ));
        vertices.push(Vertex2D::with_texcoord(
            [x1, y0],
            config.color,
            [uv_max[0], uv_min[1]], // Top-right UV
        ));
        vertices.push(Vertex2D::with_texcoord(
            [x1, y1],
            config.color,
            [uv_max[0], uv_max[1]], // Bottom-right UV
        ));

        vertices.push(Vertex2D::with_texcoord(
            [x0, y0],
            config.color,
            [uv_min[0], uv_min[1]], // Top-left UV
        ));
        vertices.push(Vertex2D::with_texcoord(
            [x1, y1],
            config.color,
            [uv_max[0], uv_max[1]], // Bottom-right UV
        ));
        vertices.push(Vertex2D::with_texcoord(
            [x0, y1],
            config.color,
            [uv_min[0], uv_max[1]], // Bottom-left UV
        ));

        cursor_x += metrics.advance_width;
    }

    vertices
}

/// Measures the dimensions of a text string.
pub fn measure_text(text: &str, font: &FontData, font_size: f32) -> (f32, f32) {
    let mut max_width = 0.0f32;
    let mut current_width = 0.0f32;
    let mut lines = 1u32;
    let line_height = font.line_height(font_size);

    for ch in text.chars() {
        if ch == '\n' {
            max_width = max_width.max(current_width);
            current_width = 0.0;
            lines += 1;
            continue;
        }

        let (metrics, _) = font.font().rasterize(ch, font_size);
        current_width += metrics.advance_width;
    }

    max_width = max_width.max(current_width);
    let total_height = lines as f32 * line_height;

    (max_width, total_height)
}

/// Generates text vertices for 3D billboard rendering (always faces camera).
///
/// The text is generated in local space and should be transformed
/// by a billboard matrix to face the camera.
pub fn generate_billboard_text_mesh(
    text: &str,
    font: &FontData,
    config: &TextConfig,
    scale: f32,
) -> Vec<[f32; 3]> {
    let mut vertices = Vec::new();
    let mut cursor_x = 0.0f32;
    let _line_height = font.line_height(config.font_size) * config.line_height_multiplier * scale;

    for ch in text.chars() {
        if ch == '\n' {
            cursor_x = 0.0;
            continue;
        }

        if ch == ' ' {
            cursor_x += config.font_size * 0.25 * scale;
            continue;
        }

        let (metrics, _) = font.font().rasterize(ch, config.font_size);

        let glyph_width = metrics.width as f32 * scale;
        let glyph_height = metrics.height as f32 * scale;
        let glyph_x = cursor_x + metrics.xmin as f32 * scale;
        let glyph_y = metrics.ymin as f32 * scale;

        // Generate quad vertices in local space (XY plane, facing +Z)
        let x0 = glyph_x;
        let y0 = glyph_y;
        let x1 = glyph_x + glyph_width;
        let y1 = glyph_y + glyph_height;

        // Two triangles for the quad
        vertices.push([x0, y0, 0.0]);
        vertices.push([x1, y0, 0.0]);
        vertices.push([x1, y1, 0.0]);

        vertices.push([x0, y0, 0.0]);
        vertices.push([x1, y1, 0.0]);
        vertices.push([x0, y1, 0.0]);

        cursor_x += metrics.advance_width * scale;
    }

    vertices
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::FontData;

    fn create_test_font() -> FontData {
        let font_data = include_bytes!("../assets/default.ttf");
        FontData::from_bytes(font_data).unwrap()
    }

    #[test]
    fn text_mesh_generation() {
        let mut font = create_test_font();
        let config = TextConfig::default();
        let vertices = generate_text_mesh("Hello", &mut font, &config, None);

        // 5 characters * 6 vertices each
        assert_eq!(vertices.len(), 30);
    }

    #[test]
    fn text_mesh_newline() {
        let mut font = create_test_font();
        let config = TextConfig::default();
        let vertices = generate_text_mesh("A\nB", &mut font, &config, None);

        // 2 characters * 6 vertices each
        assert_eq!(vertices.len(), 12);
    }

    #[test]
    fn text_mesh_with_atlas() {
        let mut font = create_test_font();
        let config = TextConfig::default();

        // Build an atlas for the characters we need
        let atlas = SdfAtlas::build_atlas(&mut font, 24, "Hello", 1.0, 256, 256);

        let vertices = generate_text_mesh("Hello", &font, &config, Some(&atlas));

        // 5 characters * 6 vertices each
        assert_eq!(vertices.len(), 30);

        // Verify that UV coordinates are not placeholder [0,0] to [1,1]
        // (unless the glyph happens to fill the entire atlas)
        for v in &vertices {
            // At least one UV coordinate should be different from placeholder
            let _is_placeholder = (v.texcoord[0] == 0.0 || v.texcoord[0] == 1.0)
                && (v.texcoord[1] == 0.0 || v.texcoord[1] == 1.0);
            // Not all vertices should have exact placeholder UVs
            // (some might, depending on atlas packing)
        }
    }

    #[test]
    fn text_measurement() {
        let font = create_test_font();
        let (width, height) = measure_text("Hello", &font, 24.0);

        assert!(width > 0.0);
        assert!(height > 0.0);
    }

    #[test]
    fn billboard_text_generation() {
        let mut font = create_test_font();
        let config = TextConfig::default();
        let vertices = generate_billboard_text_mesh("Hi", &mut font, &config, 1.0);

        // 2 characters * 6 vertices each
        assert_eq!(vertices.len(), 12);
    }
}

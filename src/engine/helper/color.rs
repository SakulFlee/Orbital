use std::num::ParseIntError;

/// Converts a 255-color RGB to 0.0-1.0 range color
pub fn rgb_to_f32_color<T: Into<u64>>(r: T, g: T, b: T) -> (f32, f32, f32) {
    (
        r.into() as f32 / 255.0,
        g.into() as f32 / 255.0,
        b.into() as f32 / 255.0,
    )
}

/// Hex to RGB
/// Expects the following format:
/// #RRGGBB
pub fn hex_to_f32_color(hex: &str) -> Result<(f32, f32, f32), ParseIntError> {
    let r = u64::from_str_radix(&hex[1..=2], 16)?;
    let g = u64::from_str_radix(&hex[3..=4], 16)?;
    let b = u64::from_str_radix(&hex[5..=6], 16)?;

    Ok(rgb_to_f32_color(r, g, b))
}

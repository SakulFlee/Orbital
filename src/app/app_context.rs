use wgpu::Color;

#[derive(Debug, Clone, Default)]
pub struct AppContext {
    pub clear_colour: Color,
    pub clear_colour_index: u32,
    pub clear_colour_increasing: bool,
}

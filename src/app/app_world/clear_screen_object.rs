use wgpu::{
    CommandBuffer, CommandEncoder, LoadOp, Operations, RenderPassColorAttachment,
    RenderPassDescriptor, TextureView,
};

use super::{object::Object, renderable::Renderable, updateable::Updateable};

const INCREASE_RATE: f64 = 0.005;

pub struct ClearScreenObject {
    clear_colour: wgpu::Color,
    clear_colour_index: u32,
    clear_colour_increasing: bool,
}

impl ClearScreenObject {
    pub fn new() -> Self {
        Self {
            clear_colour: wgpu::Color::WHITE,
            clear_colour_index: 0,
            clear_colour_increasing: true,
        }
    }
}

impl Object for ClearScreenObject {}

impl Updateable for ClearScreenObject {
    fn update(&mut self) {
        // Update colour variable
        match self.clear_colour_index {
            0 => {
                if self.clear_colour_increasing {
                    self.clear_colour.r += INCREASE_RATE;
                } else {
                    self.clear_colour.r -= INCREASE_RATE;
                }

                if self.clear_colour.r >= 1.0 || self.clear_colour.r <= 0.0 {
                    self.clear_colour_increasing = !self.clear_colour_increasing;
                }

                if self.clear_colour.r <= 0.1 && !self.clear_colour_increasing {
                    self.clear_colour_index = 1;
                    self.clear_colour_increasing = true;
                    self.clear_colour.r = 0.0;
                }
            }
            1 => {
                if self.clear_colour_increasing {
                    self.clear_colour.g += INCREASE_RATE;
                } else {
                    self.clear_colour.g -= INCREASE_RATE;
                }

                if self.clear_colour.g >= 1.0 || self.clear_colour.g <= 0.0 {
                    self.clear_colour_increasing = !self.clear_colour_increasing;
                }

                if self.clear_colour.g <= 0.1 && !self.clear_colour_increasing {
                    self.clear_colour_index = 2;
                    self.clear_colour_increasing = true;
                    self.clear_colour.g = 0.0;
                }
            }
            2 => {
                if self.clear_colour_increasing {
                    self.clear_colour.b += INCREASE_RATE;
                } else {
                    self.clear_colour.b -= INCREASE_RATE;
                }

                if self.clear_colour.b >= 1.0 || self.clear_colour.b <= 0.0 {
                    self.clear_colour_increasing = !self.clear_colour_increasing;
                }

                if self.clear_colour.b <= 0.1 && !self.clear_colour_increasing {
                    self.clear_colour_index = 0;
                    self.clear_colour_increasing = true;
                    self.clear_colour.b = 0.0;
                }
            }
            _ => (),
        }
    }
}

impl Renderable for ClearScreenObject {
    fn render(
        &mut self,
        mut command_encoder: CommandEncoder,
        texture_view: &TextureView,
    ) -> CommandBuffer {
        command_encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &texture_view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(self.clear_colour),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        command_encoder.finish()
    }

    fn do_render(&self) -> bool {
        true
    }
}

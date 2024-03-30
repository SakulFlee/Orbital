use akimo_runtime::logging::*;
use wgpu::{
    Adapter, Color, CommandEncoderDescriptor, Device, LoadOp, Operations, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, StoreOp, SurfaceConfiguration, TextureView,
};

enum Channel {
    R,
    G,
    B,
}

pub struct App {
    color: Color,
    decrement: Channel,
}

impl akimo_runtime::app::App for App {
    fn init(
        _config: &SurfaceConfiguration,
        _adapter: &Adapter,
        _device: &Device,
        _queue: &Queue,
    ) -> Self
    where
        Self: Sized,
    {
        Self {
            color: Color {
                r: 1f64,
                g: 0f64,
                b: 0f64,
                a: 1f64,
            },
            decrement: Channel::R,
        }
    }

    fn resize(&mut self, _config: &SurfaceConfiguration, _device: &Device, _queue: &Queue) {
        // Nothing needed for this example!
        // Later, this should be used to update the uniform buffer matrix.
    }

    fn update(&mut self) {
        // Nothing needed for this example!
        // All events that we care about are already taken care of.

        const INTERVAL: f64 = 0.001f64;

        match self.decrement {
            Channel::R => {
                self.color.r -= INTERVAL;
                self.color.g += INTERVAL;

                if self.color.r <= INTERVAL {
                    self.color.r = 0.0f64;
                    self.color.g = 1.0f64;
                    self.decrement = Channel::G;
                }
            }
            Channel::G => {
                self.color.g -= INTERVAL;
                self.color.b += INTERVAL;

                if self.color.g <= INTERVAL {
                    self.color.g = 0.0f64;
                    self.color.b = 1.0f64;

                    self.decrement = Channel::B;
                }
            }
            Channel::B => {
                self.color.b -= INTERVAL;
                self.color.r += INTERVAL;

                if self.color.b <= INTERVAL {
                    self.color.b = 0.0f64;
                    self.color.r = 1.0f64;

                    self.decrement = Channel::R;
                }
            }
        }
    }

    fn render(&mut self, view: &TextureView, device: &Device, queue: &Queue) {
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: None });

        {
            let mut _render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(self.color),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }

        queue.submit(Some(encoder.finish()));
    }
}

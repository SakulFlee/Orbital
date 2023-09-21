use std::iter::once;

use wgpu::{
    Color, CommandEncoderDescriptor, LoadOp, Operations, RenderPassColorAttachment,
    RenderPassDescriptor,
};
use wgpu_engine::{
    engine::{TComputingEngine, TRenderingEngine, TextureHelper, WGPURenderingEngine},
    log::log_init,
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    // Log initialization
    log_init();

    // App
    // let app = App::from_app_config_default_path();
    // app.hijack_thread_and_run().await;

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .build(&event_loop)
        .expect("Window creation failed");

    let engine = WGPURenderingEngine::new(&window).expect("Engine creation failed");

    event_loop.run(move |event, _target, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            Event::RedrawEventsCleared => window.request_redraw(),
            Event::RedrawRequested(..) => {
                // Note: DO NOT concat like `engine.get_surface_texture().unwrap().make_texture_view()`!
                // It drops _something_ which makes WGPU crash!
                // Furthermore, below `queue.submit(...)` DO NOT call `engine.get_surface_texture()` again, but use THIS reference.
                // If called, a 2nd image is being acquired which, once again, crashes WGPU
                let surface_texture = engine.get_surface_texture().expect("!");
                let surface_texture_view = surface_texture.make_texture_view();

                let mut command_encoder =
                    engine
                        .get_device()
                        .create_command_encoder(&CommandEncoderDescriptor {
                            label: Some("Command Encoder"),
                        });

                {
                    let _render_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
                        label: Some("Render Pass"),
                        color_attachments: &[Some(RenderPassColorAttachment {
                            view: &surface_texture_view,
                            resolve_target: None,
                            ops: Operations {
                                load: LoadOp::Clear(Color {
                                    // Sky blue - ish
                                    r: 0.0,
                                    g: 0.61176,
                                    b: 0.77647,
                                    a: 1.0,
                                }),
                                store: true,
                            },
                        })],
                        depth_stencil_attachment: None, // TODO
                    });

                    // render_pass.set_pipeline(engine.get_render_pipeline());
                }

                let command_buffer = command_encoder.finish();
                engine.get_queue().submit(once(command_buffer));
                // Note: DO NOT recall `engine.get_surface*`. See note above.
                surface_texture.present();
            }
            _ => (),
        }
    });
}

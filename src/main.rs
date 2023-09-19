use wgpu_engine::{log_init};
use winit::{event_loop::EventLoop, window::WindowBuilder};

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

    // let command_encoder = engine
    //     .get_device()
    //     .create_command_encoder(&CommandEncoderDescriptor {
    //         label: Some("Command Encoder"),
    //     });
    // let render_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
    //     label: Some("Render Pass"),
    //     color_attachments: &[Some(RenderPassColorAttachment {
    //         view: engine.get_surface_texture_view(),
    //         resolve_target: None,
    //         ops: Operations {
    //             load: LoadOp::Clear(Color {
    //                 // Sky blue - ish
    //                 r: 0.0,
    //                 g: 0.61176,
    //                 b: 0.77647,
    //                 a: 1.0,
    //             }),
    //             store: true,
    //         },
    //     })],
    //     depth_stencil_attachment: None, // TODO
    // });

    // render_pass.set_pipeline(engine.get_render_pipeline());
    // // render_pass.draw(vertices, instances)
}

use wgpu_engine::{
    app::{App, World},
    log::log_init,
};

fn main() {
    // Log initialization
    log_init();

    let world = World::new();
    App::run("WGPU", world).expect("App failed");

    //         Event::RedrawRequested(..) => {
    //             // Note: DO NOT concat like `engine.get_surface_texture().unwrap().make_texture_view()`!
    //             // It drops _something_ which makes WGPU crash!
    //             // Furthermore, below `queue.submit(...)` DO NOT call `engine.get_surface_texture()` again, but use THIS reference.
    //             // If called, a 2nd image is being acquired which, once again, crashes WGPU
    //             let surface_texture = engine.get_surface_texture().expect("!");
    //             let surface_texture_view = surface_texture.make_texture_view();

    //             let mut command_encoder =
    //                 engine
    //                     .get_device()
    //                     .create_command_encoder(&CommandEncoderDescriptor {
    //                         label: Some("Command Encoder"),
    //                     });

    //             {
    //                 let _render_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
    //                     label: Some("Render Pass"),
    //                     color_attachments: &[Some(RenderPassColorAttachment {
    //                         view: &surface_texture_view,
    //                         resolve_target: None,
    //                         ops: Operations {
    //                             load: LoadOp::Clear(Color {
    //                                 // Sky blue - ish
    //                                 r: 0.0,
    //                                 g: 0.61176,
    //                                 b: 0.77647,
    //                                 a: 1.0,
    //                             }),
    //                             store: true,
    //                         },
    //                     })],
    //                     depth_stencil_attachment: None, // TODO
    //                 });

    //                 // render_pass.set_pipeline(engine.get_render_pipeline());
    //             }

    //             let command_buffer = command_encoder.finish();
    //             engine.get_queue().submit(once(command_buffer));
    //             // Note: DO NOT recall `engine.get_surface*`. See note above.
    //             surface_texture.present();
    //         }
    //         _ => (),
    //     }
    // });
}

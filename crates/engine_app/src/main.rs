use compute_engine_wgpu::ComputeEngineWGPU;
use engine_result_wgpu::EngineResultWGPU;
use event_system::event_system;
use logging::*;

#[tokio::main]
async fn main() -> EngineResultWGPU<()> {
    logging::log_init();

    info!("Akimo-Project: Engine");
    info!("(C) SakulFlee 2024");

    let compute_engine = ComputeEngineWGPU::new().await?;

    // let event_loop = EventLoop::new().expect("EventLoop boot failed");
    // let window = WindowBuilder::new()
    //     .with_canvas(Some(canvas))
    //     .build(&event_loop)
    //     .expect("Winit window/canvas creation failed");

    // let compute_engine = ComputeEngineWGPU::new().await;
    // debug!("CE: {:?}", compute_engine);

    // event_loop.set_control_flow(event_loop::ControlFlow::Poll);
    // event_loop
    //     .run(move |event, window_target| {
    //         match event {
    //             Event::WindowEvent {
    //                 event: WindowEvent::CloseRequested,
    //                 ..
    //             } => {
    //                 println!("The close button was pressed; stopping");
    //                 window_target.exit();
    //             }
    //             Event::AboutToWait => {
    //                 // Application update code.

    //                 // Queue a RedrawRequested event.
    //                 //
    //                 // You only need to call this if you've determined that you need to redraw in
    //                 // applications which do not always need to. Applications that redraw continuously
    //                 // can render here instead.
    //                 window.request_redraw();
    //             }
    //             Event::WindowEvent {
    //                 event: WindowEvent::RedrawRequested,
    //                 ..
    //             } => {
    //                 // Redraw the application.
    //                 //
    //                 // It's preferable for applications that do not render continuously to render in
    //                 // this event rather than in AboutToWait, since rendering in here allows
    //                 // the program to gracefully handle redraws requested by the OS.

    //                 debug!("REDRAW!!!");
    //             }
    //             _ => (),
    //         }
    //     })
    //     .expect("EventLoop failed to run!");

    Ok(())
}

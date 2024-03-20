use logging::*;
use winit::{dpi::LogicalSize, event_loop::EventLoop, window::WindowBuilder};

#[tokio::main]
async fn main() -> EngineResultWGPU<()> {
    logging::log_init();

    info!("Akimo-Project: Engine");
    info!("(C) SakulFlee 2024");

    let size = LogicalSize::new(1280, 720);

    let event_loop = EventLoop::new().expect("EventLoop boot failed");
    let window = WindowBuilder::new()
        .with_inner_size(size)
        .build(&event_loop)
        .expect("Winit window/canvas creation failed");

    // TODO
    let surface = gra.instance().create_surface(&window);

    // let _connector = GraphicConnectorWGPU::from_surface(window., size).await?;

    // let compute_connector = ComputeConnectorWGPU::new().await;
    // debug!("CE: {:?}", compute_connector);

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

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

pub struct App {
    event_loop: EventLoop<()>,
    window: Window,
}

impl App {
    pub fn new(control_flow: ControlFlow) -> Self {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(control_flow);

        let window = Window::new(&event_loop).unwrap();

        Self { event_loop, window }
    }

    pub fn run(self) {
        self.event_loop
            .run(move |event, elwt| {
                match event {
                    Event::WindowEvent {
                        event: WindowEvent::CloseRequested,
                        ..
                    } => {
                        println!("The close button was pressed; stopping");
                        elwt.exit();
                    }
                    Event::AboutToWait => {
                        // Application update code.

                        // Queue a RedrawRequested event.
                        //
                        // You only need to call this if you've determined that you need to redraw in
                        // applications which do not always need to. Applications that redraw continuously
                        // can render here instead.
                        self.window.request_redraw();
                    }
                    Event::WindowEvent {
                        event: WindowEvent::RedrawRequested,
                        ..
                    } => {
                        // Redraw the application.
                        //
                        // It's preferable for applications that do not render continuously to render in
                        // this event rather than in AboutToWait, since rendering in here allows
                        // the program to gracefully handle redraws requested by the OS.
                        println!("Redraw!");
                    }
                    _ => (),
                }
            })
            .unwrap();
    }
}

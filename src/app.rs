use winit::event::WindowEvent;
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

use crate::Scene;

/// # Application
///
/// Entry-point for building a Pulse application.
pub trait Application: Sized {
    /// Returns the title to be displayed in the application window.
    fn title(&self) -> &str;

    /// Returns the current state of the application. The application will exit if this returns
    /// [ApplicationState::Finished] after [Application::handle_event] or [Application::update] is
    /// called.
    fn state(&self) -> ApplicationState;

    /// Handles the incoming event.
    fn handle_event(&mut self, event: Event);

    /// Updates the application for the current frame.
    fn update(&mut self);

    /// Returns a reference to the application's scene.
    fn scene(&self) -> &Scene;

    /// Runs the application.
    fn run(self) {
        run_application(self);
    }
}

/// # Application State
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ApplicationState {
    /// Application is running.
    Running,
    /// Application has finished running.
    Finished,
}

/// # Event
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Event {
    /// Application window requested to close.
    CloseRequested,
}

fn run_application(mut app: impl Application) {
    let event_loop = EventLoop::new().unwrap();
    let mut window_title = app.title().to_string();
    let window = WindowBuilder::new()
        .with_title(&window_title)
        .build(&event_loop)
        .unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop
        .run(|event, event_loop_window_target| {
            match event {
                winit::event::Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => {
                        app.handle_event(Event::CloseRequested);
                    }
                    _ => {}
                },
                winit::event::Event::AboutToWait => {
                    app.update();

                    let title = app.title();
                    if title != &window_title {
                        window_title = title.to_string();
                        window.set_title(&window_title);
                    }
                }
                _ => {}
            }

            if app.state() == ApplicationState::Finished {
                event_loop_window_target.exit();
            }
        })
        .unwrap();
}

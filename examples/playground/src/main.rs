use pulse::Application;
use pulse::ApplicationState;
use pulse::Event;
use pulse::Scene;

struct Playground {
    state: ApplicationState,
    scene: Scene,
}

impl Playground {
    fn new() -> Self {
        Self {
            state: ApplicationState::Running,
            scene: Scene::new(),
        }
    }
}

impl Application for Playground {
    fn title(&self) -> &str {
        "Pulse Playground"
    }

    fn state(&self) -> ApplicationState {
        self.state
    }

    fn handle_event(&mut self, event: Event) {
        match event {
            Event::CloseRequested => {
                self.state = ApplicationState::Finished;
            }
        }
    }

    fn update(&mut self) {}

    fn scene(&self) -> &Scene {
        &self.scene
    }
}

fn main() {
    Playground::new().run();
}

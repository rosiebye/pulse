use pulse::Application;
use pulse::ApplicationState;
use pulse::Event;
use pulse::LocalTransform;
use pulse::Scene;
use pulse::Visibility;

struct Playground {
    state: ApplicationState,
    scene: Scene,
}

impl Playground {
    fn new() -> Self {
        let mut scene = Scene::new();

        let node = scene.spawn();
        scene.add(node, Visibility::Visible);
        scene.add(node, LocalTransform::IDENTITY);

        Self {
            state: ApplicationState::Running,
            scene,
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

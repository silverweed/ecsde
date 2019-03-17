use sfml::graphics as sfgfx;
use sfml::window as sfwin;
use std::vec::Vec;

#[derive(PartialEq, Hash)]
pub enum Action {
    Quit,
    Resize(u32, u32),
}

pub struct Input_System {
    actions: Vec<Action>,
}

impl Input_System {
    pub fn new() -> Input_System {
        Input_System { actions: vec![] }
    }

    pub fn has_action(&self, action: &Action) -> bool {
        self.actions.contains(&action)
    }

    pub fn get_actions(&self) -> &Vec<Action> {
        &self.actions
    }

    pub fn update(&mut self, window: &mut sfgfx::RenderWindow) {
        use sfwin::Key;

        self.actions.clear();

        while let Some(event) = window.poll_event() {
            match event {
                sfwin::Event::Closed => self.actions.push(Action::Quit),
                sfwin::Event::KeyPressed { code, .. } => match code {
                    Key::Q => self.actions.push(Action::Quit),
                    _ => (),
                },
                sfwin::Event::Resized { width, height } => {
                    self.actions.push(Action::Resize(width, height))
                }
                _ => (),
            }
        }
    }
}

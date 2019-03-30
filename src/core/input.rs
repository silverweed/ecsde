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

    pub fn update(&mut self, event_pump: &mut sdl2::EventPump) {
        use sdl2::event::{Event, WindowEvent};
        use sdl2::keyboard::Keycode;

        self.actions.clear();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => self.actions.push(Action::Quit),
                Event::KeyDown { keycode: code, .. } => match code {
                    Some(Keycode::Q) => self.actions.push(Action::Quit),
                    _ => (),
                },
                Event::Window {
                    win_event: WindowEvent::Resized(width, height),
                    ..
                } => self
                    .actions
                    .push(Action::Resize(width as u32, height as u32)),
                _ => (),
            }
        }
    }
}

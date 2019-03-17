use super::system::System;
use crate::gfx::window::Window;
use sfml::graphics as sfgfx;
use sfml::window as sfwin;
use std::cell::RefCell;
use std::rc::Rc;
use std::time;
use std::vec::Vec;

#[derive(PartialEq, Hash)]
pub enum Action {
    Quit,
    Resize(u32, u32),
}

impl Eq for Action {}

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
}

pub struct Input_System_Update_Params {
    pub window: Rc<RefCell<Window>>,
}

impl System for Input_System {
    type Config = ();
    type Update_Params = Input_System_Update_Params;

    fn update(&mut self, params: Self::Update_Params) {
        use sfwin::Key;

        self.actions.clear();

        let window = &mut params.window.borrow_mut().sf_win;
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

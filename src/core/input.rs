use super::common;
use super::system::System;
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
    // Note: we need mutable access to window for polling events,
    // but the window is also mutably owned by the rendering system.
    window: Rc<RefCell<sfwin::Window>>,
    actions: Vec<Action>,
}

impl Input_System {
    pub fn new(window: Rc<RefCell<sfwin::Window>>) -> Input_System {
        Input_System {
            window: Rc::clone(&window),
            actions: vec![],
        }
    }
}

impl System for Input_System {
    type Config = ();

    fn update(&mut self, _delta: &time::Duration) {
        use sfwin::Key;

        self.actions.clear();

        while let Some(event) = self.window.borrow_mut().poll_event() {
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

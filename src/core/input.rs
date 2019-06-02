use crate::core::common::direction::Direction;
use crate::core::common::vector::Vec2f;
use cgmath::InnerSpace;
use std::sync::mpsc::{channel, Sender};
use std::vec::Vec;

#[derive(PartialEq, Hash, Copy, Clone)]
pub enum Action {
    Quit,
    Resize(u32, u32),
    Move(Direction),
    Zoom(i32), // Note: the zoom factor is an integer rather than a float as it can be hashed.
    // This integer must be divided by 100 to obtain the actual scaling factor.
    Change_Speed(i32),
    Pause_Toggle,
    Step_Simulation,
}

#[derive(Default, Clone)]
pub struct Action_List {
    quit: bool,
    move_up: bool,
    move_left: bool,
    move_down: bool,
    move_right: bool,
    actions: Vec<Action>,
}

impl Action_List {
    pub fn has_action(&self, action: &Action) -> bool {
        match action {
            Action::Quit => self.quit,
            Action::Move(Direction::Up) => self.move_up,
            Action::Move(Direction::Left) => self.move_left,
            Action::Move(Direction::Down) => self.move_down,
            Action::Move(Direction::Right) => self.move_right,
            _ => self.actions.contains(&action),
        }
    }
}

impl std::ops::Deref for Action_List {
    type Target = <std::vec::Vec<Action> as std::ops::Deref>::Target;

    fn deref(&self) -> &Self::Target {
        self.actions.deref()
    }
}

pub struct Input_System {
    actions: Action_List,
}

impl Input_System {
    pub fn new(actions_tx: Sender<Action_List>) -> Input_System {
        Input_System {
            actions: Action_List::default(),
        }
    }

    pub fn get_action_list(&self) -> Action_List {
        self.actions.clone()
    }

    #[cfg(feature = "use-sdl")]
    pub fn update(&mut self, event_pump: &mut sdl2::EventPump) {
        poll_events(&mut self.actions, event_pump);
    }

    #[cfg(feature = "use-sfml")]
    pub fn update(&mut self, window: &mut sfml::graphics::RenderWindow) {
        poll_events(&mut self.actions, window);
    }
}

pub fn get_movement_from_input(actions: &Action_List) -> Vec2f {
    use crate::core::common::direction::Direction;

    let mut movement = Vec2f::new(0.0, 0.0);
    if actions.has_action(&Action::Move(Direction::Left)) {
        movement.x -= 1.0;
    }
    if actions.has_action(&Action::Move(Direction::Right)) {
        movement.x += 1.0;
    }
    if actions.has_action(&Action::Move(Direction::Up)) {
        movement.y -= 1.0;
    }
    if actions.has_action(&Action::Move(Direction::Down)) {
        movement.y += 1.0;
    }
    movement
}

pub fn get_normalized_movement_from_input(actions: &Action_List) -> Vec2f {
    let m = get_movement_from_input(actions);
    if m.magnitude2() == 0.0 {
        m
    } else {
        m.normalize()
    }
}

#[cfg(feature = "use-sdl")]
fn poll_events(action_list: &mut Action_List, event_pump: &mut sdl2::EventPump) {
    use sdl2::event::{Event, WindowEvent};
    use sdl2::keyboard::Keycode;

    let actions = &mut action_list.actions;
    actions.clear();

    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. }
            | Event::KeyDown {
                keycode: Some(Keycode::Q),
                ..
            } => action_list.quit = true,
            Event::KeyDown {
                keycode: Some(keycode),
                ..
            } => match keycode {
                Keycode::W => action_list.move_up = true,
                Keycode::A => action_list.move_left = true,
                Keycode::S => action_list.move_down = true,
                Keycode::D => action_list.move_right = true,
                Keycode::KpPlus => actions.push(Action::Zoom(10)),
                Keycode::KpMinus => actions.push(Action::Zoom(-10)),
                Keycode::Num1 | Keycode::Minus => actions.push(Action::Change_Speed(-10)),
                Keycode::Num2 | Keycode::Equals => actions.push(Action::Change_Speed(10)),
                Keycode::Period => actions.push(Action::Pause_Toggle),
                Keycode::Slash => actions.push(Action::Step_Simulation),
                _ => (),
            },
            Event::KeyUp {
                keycode: Some(keycode),
                ..
            } => match keycode {
                Keycode::W => action_list.move_up = false,
                Keycode::A => action_list.move_left = false,
                Keycode::S => action_list.move_down = false,
                Keycode::D => action_list.move_right = false,
                _ => (),
            },
            Event::Window {
                win_event: WindowEvent::Resized(width, height),
                ..
            } => actions.push(Action::Resize(width as u32, height as u32)),
            _ => (),
        }
    }
}

#[cfg(feature = "use-sfml")]
fn poll_events(action_list: &mut Action_List, window: &mut sfml::graphics::RenderWindow) {
    use sfml::window::{Event, Key};

    let actions = &mut action_list.actions;
    actions.clear();

    while let Some(event) = window.poll_event() {
        match event {
            Event::Closed { .. } | Event::KeyPressed { code: Key::Q, .. } => {
                action_list.quit = true
            }
            Event::KeyPressed { code, .. } => match code {
                Key::W => action_list.move_up = true,
                Key::A => action_list.move_left = true,
                Key::S => action_list.move_down = true,
                Key::D => action_list.move_right = true,
                //Key::KpPlus => actions.push(Action::Zoom(10)),
                //Key::KpMinus => actions.push(Action::Zoom(-10)),
                Key::Num1 | Key::Dash => actions.push(Action::Change_Speed(-10)),
                Key::Num2 | Key::Equal => actions.push(Action::Change_Speed(10)),
                Key::Period => actions.push(Action::Pause_Toggle),
                Key::Slash => actions.push(Action::Step_Simulation),
                _ => (),
            },
            Event::KeyReleased { code, .. } => match code {
                Key::W => action_list.move_up = false,
                Key::A => action_list.move_left = false,
                Key::S => action_list.move_down = false,
                Key::D => action_list.move_right = false,
                _ => (),
            },
            Event::Resized { width, height } => actions.push(Action::Resize(width, height)),
            _ => (),
        }
    }
}

use crate::core::common::direction::Direction;
use crate::core::common::vector::Vec2f;
use std::vec::Vec;

#[derive(PartialEq, Hash)]
pub enum Action {
    Quit,
    Resize(u32, u32),
    Move(Direction),
    Zoom(i32), // Note: the zoom factor is an integer rather than a float as it can be hashed.
    // This integer must be divided by 100 to obtain the actual scaling factor.
    ChangeSpeed(i32),
}

#[derive(Default)]
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
    pub fn new() -> Input_System {
        Input_System {
            actions: Action_List::default(),
        }
    }

    pub fn get_actions(&self) -> &Action_List {
        &self.actions
    }

    pub fn update(&mut self, event_pump: &mut sdl2::EventPump) {
        use sdl2::event::{Event, WindowEvent};
        use sdl2::keyboard::Keycode;

        let actions = &mut self.actions.actions;
        actions.clear();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Q),
                    ..
                } => self.actions.quit = true,
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => match keycode {
                    Keycode::W => self.actions.move_up = true,
                    Keycode::A => self.actions.move_left = true,
                    Keycode::S => self.actions.move_down = true,
                    Keycode::D => self.actions.move_right = true,
                    Keycode::KpPlus => actions.push(Action::Zoom(10)),
                    Keycode::KpMinus => actions.push(Action::Zoom(-10)),
                    Keycode::Num1 => actions.push(Action::ChangeSpeed(-10)),
                    Keycode::Num2 => actions.push(Action::ChangeSpeed(10)),
                    _ => (),
                },
                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => match keycode {
                    Keycode::W => self.actions.move_up = false,
                    Keycode::A => self.actions.move_left = false,
                    Keycode::S => self.actions.move_down = false,
                    Keycode::D => self.actions.move_right = false,
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

use crate::core::common::direction::{Direction, Direction_Flags};
use crate::core::common::vector::Vec2f;
use crate::replay::replay_data::Replay_Data_Iter;
use cgmath::InnerSpace;
use std::vec::Vec;

#[derive(PartialEq, Hash, Copy, Clone)]
pub enum Action {
    Quit,
    Resize(u32, u32),
    Move(Direction),
    // Note: the zoom factor is an integer rather than a float as it can be hashed.
    // This integer must be divided by 100 to obtain the actual scaling factor.
    Zoom(i32),
    Change_Speed(i32),
    Pause_Toggle,
    Step_Simulation,
    Print_Entity_Manager_Debug_Info,
}

#[derive(Default, Clone)]
pub struct Action_List {
    quit: bool,
    directions: Direction_Flags,
    actions: Vec<Action>,
}

impl Action_List {
    pub fn has_action(&self, action: &Action) -> bool {
        match action {
            Action::Quit => self.quit,
            Action::Move(Direction::Up) => self.directions.contains(Direction_Flags::UP),
            Action::Move(Direction::Left) => self.directions.contains(Direction_Flags::LEFT),
            Action::Move(Direction::Down) => self.directions.contains(Direction_Flags::DOWN),
            Action::Move(Direction::Right) => self.directions.contains(Direction_Flags::RIGHT),
            _ => self.actions.contains(&action),
        }
    }

    pub fn get_directions(&self) -> Direction_Flags {
        self.directions
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

    pub fn update_from_replay(
        &mut self,
        cur_frame: u64,
        replay_data_iter: &mut Replay_Data_Iter<'_>,
    ) {
        if let Some(datum) = replay_data_iter.next() {
            if datum.frame_number() >= cur_frame {
                self.actions.directions = datum.directions();
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

pub fn get_normalized_movement_from_input(actions: &Action_List) -> Vec2f {
    let m = get_movement_from_input(actions);
    if m.magnitude2() == 0.0 {
        m
    } else {
        m.normalize()
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
                Key::W => action_list.directions.insert(Direction_Flags::UP),
                Key::A => action_list.directions.insert(Direction_Flags::LEFT),
                Key::S => action_list.directions.insert(Direction_Flags::DOWN),
                Key::D => action_list.directions.insert(Direction_Flags::RIGHT),
                //Key::KpPlus => actions.push(Action::Zoom(10)),
                //Key::KpMinus => actions.push(Action::Zoom(-10)),
                Key::Num1 | Key::Dash => actions.push(Action::Change_Speed(-10)),
                Key::Num2 | Key::Equal => actions.push(Action::Change_Speed(10)),
                Key::Period => actions.push(Action::Pause_Toggle),
                Key::Slash => actions.push(Action::Step_Simulation),
                Key::M => actions.push(Action::Print_Entity_Manager_Debug_Info),
                _ => (),
            },
            Event::KeyReleased { code, .. } => match code {
                Key::W => action_list.directions.remove(Direction_Flags::UP),
                Key::A => action_list.directions.remove(Direction_Flags::LEFT),
                Key::S => action_list.directions.remove(Direction_Flags::DOWN),
                Key::D => action_list.directions.remove(Direction_Flags::RIGHT),
                _ => (),
            },
            Event::Resized { width, height } => actions.push(Action::Resize(width, height)),
            _ => (),
        }
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
                Keycode::M => actions.push(Action::Print_Entity_Manager_Debug_Info),
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

use super::actions::{Action, Action_List};
use super::bindings::{Action_Kind, Action_Mappings, Input_Bindings};
use crate::core::common::direction::Direction_Flags;
use crate::core::common::stringid::String_Id;
use crate::core::env::Env_Info;
use crate::replay::replay_data::Replay_Data_Iter;

pub struct Input_System {
    actions: Action_List,
    bindings: Input_Bindings,
    action_mappings: Action_Mappings,
}

impl Input_System {
    pub fn new(env: &Env_Info) -> Input_System {
        Input_System {
            actions: Action_List::default(),
            bindings: super::create_bindings(env),
            action_mappings: Action_Mappings::new(),
        }
    }

    pub fn init(&mut self) {
        // @Temporary: move most of these bindings somewhere else.
        // We certainly don't want things like "print_em_debug_info" or "move_up" in such a
        // core part of the engine!
        self.action_mappings.register_mapping(
            String_Id::from("quit"),
            Action_Kind::Pressed,
            Box::new(|actions| actions.actions.push(Action::Quit)),
        );
        self.action_mappings.register_mapping(
            String_Id::from("game_speed_up"),
            Action_Kind::Pressed,
            Box::new(|actions| actions.actions.push(Action::Change_Speed(10))),
        );
        self.action_mappings.register_mapping(
            String_Id::from("game_speed_down"),
            Action_Kind::Pressed,
            Box::new(|actions| actions.actions.push(Action::Change_Speed(-10))),
        );
        self.action_mappings.register_mapping(
            String_Id::from("pause_toggle"),
            Action_Kind::Pressed,
            Box::new(|actions| actions.actions.push(Action::Pause_Toggle)),
        );
        self.action_mappings.register_mapping(
            String_Id::from("print_em_debug_info"),
            Action_Kind::Pressed,
            Box::new(|actions| {
                actions
                    .actions
                    .push(Action::Print_Entity_Manager_Debug_Info)
            }),
        );
        self.action_mappings.register_mapping(
            String_Id::from("step_sim"),
            Action_Kind::Pressed,
            Box::new(|actions| actions.actions.push(Action::Step_Simulation)),
        );
        self.action_mappings.register_mapping(
            String_Id::from("move_up"),
            Action_Kind::Pressed,
            Box::new(|actions| actions.directions.insert(Direction_Flags::UP)),
        );
        self.action_mappings.register_mapping(
            String_Id::from("move_left"),
            Action_Kind::Pressed,
            Box::new(|actions| actions.directions.insert(Direction_Flags::LEFT)),
        );
        self.action_mappings.register_mapping(
            String_Id::from("move_down"),
            Action_Kind::Pressed,
            Box::new(|actions| actions.directions.insert(Direction_Flags::DOWN)),
        );
        self.action_mappings.register_mapping(
            String_Id::from("move_right"),
            Action_Kind::Pressed,
            Box::new(|actions| actions.directions.insert(Direction_Flags::RIGHT)),
        );
        self.action_mappings.register_mapping(
            String_Id::from("move_up"),
            Action_Kind::Released,
            Box::new(|actions| actions.directions.remove(Direction_Flags::UP)),
        );
        self.action_mappings.register_mapping(
            String_Id::from("move_left"),
            Action_Kind::Released,
            Box::new(|actions| actions.directions.remove(Direction_Flags::LEFT)),
        );
        self.action_mappings.register_mapping(
            String_Id::from("move_down"),
            Action_Kind::Released,
            Box::new(|actions| actions.directions.remove(Direction_Flags::DOWN)),
        );
        self.action_mappings.register_mapping(
            String_Id::from("move_right"),
            Action_Kind::Released,
            Box::new(|actions| actions.directions.remove(Direction_Flags::RIGHT)),
        );
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
        self.poll_events(window);
    }

    /// Returns true as long as the replay isn't over
    pub fn update_from_replay(
        &mut self,
        cur_frame: u64,
        replay_data_iter: &mut Replay_Data_Iter<'_>,
    ) -> bool {
        if let Some(datum) = replay_data_iter.cur() {
            if cur_frame >= datum.frame_number() {
                self.actions.directions = datum.directions();
                replay_data_iter.next().is_some()
            } else {
                true
            }
        } else {
            false
        }
    }

    #[cfg(feature = "use-sfml")]
    fn poll_events(&mut self, window: &mut sfml::graphics::RenderWindow) {
        use sfml::window::Event;

        let bindings = &self.bindings;
        self.actions.actions.clear();

        let handle_actions = |actions: &mut Action_List,
                              kind: Action_Kind,
                              names: &Vec<_>,
                              mappings: &Action_Mappings| {
            for name in names.iter() {
                if let Some(callbacks) = mappings.get_callbacks_for_action(*name, kind) {
                    for callback in callbacks.iter() {
                        callback(actions);
                    }
                }
            }
        };

        while let Some(event) = window.poll_event() {
            match event {
                Event::Closed { .. } => self.actions.actions.push(Action::Quit),
                Event::KeyPressed { code, .. } => {
                    if let Some(action_names) = bindings.get_key_action(code) {
                        handle_actions(
                            &mut self.actions,
                            Action_Kind::Pressed,
                            action_names,
                            &self.action_mappings,
                        );
                    }
                }
                Event::KeyReleased { code, .. } => {
                    if let Some(action_names) = bindings.get_key_action(code) {
                        handle_actions(
                            &mut self.actions,
                            Action_Kind::Released,
                            action_names,
                            &self.action_mappings,
                        );
                    }
                }
                Event::JoystickButtonPressed { joystickid, button } => {
                    if let Some(action_names) = bindings.get_joystick_action(joystickid, button) {
                        handle_actions(
                            &mut self.actions,
                            Action_Kind::Pressed,
                            action_names,
                            &self.action_mappings,
                        );
                    }
                }
                Event::JoystickButtonReleased { joystickid, button } => {
                    if let Some(action_names) = bindings.get_joystick_action(joystickid, button) {
                        handle_actions(
                            &mut self.actions,
                            Action_Kind::Released,
                            action_names,
                            &self.action_mappings,
                        );
                    }
                }
                Event::MouseButtonPressed { button, .. } => {
                    if let Some(action_names) = bindings.get_mouse_action(button) {
                        handle_actions(
                            &mut self.actions,
                            Action_Kind::Pressed,
                            action_names,
                            &self.action_mappings,
                        );
                    }
                }
                Event::MouseButtonReleased { button, .. } => {
                    if let Some(action_names) = bindings.get_mouse_action(button) {
                        handle_actions(
                            &mut self.actions,
                            Action_Kind::Released,
                            action_names,
                            &self.action_mappings,
                        );
                    }
                }
                _ => (),
            }
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
            } => actions.push(Action::Quit),
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

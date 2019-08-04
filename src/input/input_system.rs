use super::actions::{Action, Action_List};
use super::bindings::Input_Bindings;
use super::callbacks::{Action_Callbacks, Action_Kind};
use super::provider::{Input_Provider, Input_Provider_Input, Input_Provider_Output};
use crate::core::common::direction::Direction_Flags;
use crate::core::common::stringid::String_Id;
use crate::core::env::Env_Info;

pub struct Default_Input_Provider {}

impl Input_Provider for Default_Input_Provider {
    fn poll_events(&mut self, window: &mut Input_Provider_Input) -> Vec<Input_Provider_Output> {
        let mut events = vec![];
        while let Some(evt) = window.poll_event() {
            events.push(evt);
        }
        events
    }

    fn is_realtime_player_input(&self) -> bool {
        true
    }
}

pub struct Input_System {
    // @Incomplete: refactor me!
    actions: Action_List,
    bindings: Input_Bindings,
    action_callbacks: Action_Callbacks,
}

impl Input_System {
    pub fn new(env: &Env_Info) -> Input_System {
        Input_System {
            actions: Action_List::default(),
            bindings: super::create_bindings(env),
            action_callbacks: Action_Callbacks::new(),
        }
    }

    pub fn init(&mut self) {
        // @Temporary: move most of these bindings somewhere else.
        // We certainly don't want things like "print_em_debug_info" or "move_up" in such a
        // core part of the engine!
        self.action_callbacks.register_mapping(
            String_Id::from("quit"),
            Action_Kind::Pressed,
            |actions| actions.actions.push(Action::Quit),
        );
        self.action_callbacks.register_mapping(
            String_Id::from("game_speed_up"),
            Action_Kind::Pressed,
            |actions| actions.actions.push(Action::Change_Speed(10)),
        );
        self.action_callbacks.register_mapping(
            String_Id::from("game_speed_down"),
            Action_Kind::Pressed,
            |actions| actions.actions.push(Action::Change_Speed(-10)),
        );
        self.action_callbacks.register_mapping(
            String_Id::from("pause_toggle"),
            Action_Kind::Pressed,
            |actions| actions.actions.push(Action::Pause_Toggle),
        );
        self.action_callbacks.register_mapping(
            String_Id::from("print_em_debug_info"),
            Action_Kind::Pressed,
            |actions| {
                actions
                    .actions
                    .push(Action::Print_Entity_Manager_Debug_Info)
            },
        );
        self.action_callbacks.register_mapping(
            String_Id::from("step_sim"),
            Action_Kind::Pressed,
            |actions| actions.actions.push(Action::Step_Simulation),
        );
        self.action_callbacks.register_mapping(
            String_Id::from("move_up"),
            Action_Kind::Pressed,
            |actions| actions.directions.insert(Direction_Flags::UP),
        );
        self.action_callbacks.register_mapping(
            String_Id::from("move_left"),
            Action_Kind::Pressed,
            |actions| actions.directions.insert(Direction_Flags::LEFT),
        );
        self.action_callbacks.register_mapping(
            String_Id::from("move_down"),
            Action_Kind::Pressed,
            |actions| actions.directions.insert(Direction_Flags::DOWN),
        );
        self.action_callbacks.register_mapping(
            String_Id::from("move_right"),
            Action_Kind::Pressed,
            |actions| actions.directions.insert(Direction_Flags::RIGHT),
        );
        self.action_callbacks.register_mapping(
            String_Id::from("move_up"),
            Action_Kind::Released,
            |actions| actions.directions.remove(Direction_Flags::UP),
        );
        self.action_callbacks.register_mapping(
            String_Id::from("move_left"),
            Action_Kind::Released,
            |actions| actions.directions.remove(Direction_Flags::LEFT),
        );
        self.action_callbacks.register_mapping(
            String_Id::from("move_down"),
            Action_Kind::Released,
            |actions| actions.directions.remove(Direction_Flags::DOWN),
        );
        self.action_callbacks.register_mapping(
            String_Id::from("move_right"),
            Action_Kind::Released,
            |actions| actions.directions.remove(Direction_Flags::RIGHT),
        );
    }

    pub fn get_action_list(&self) -> Action_List {
        self.actions.clone()
    }

    #[cfg(feature = "use-sfml")]
    pub fn update(
        &mut self,
        window: &mut sfml::graphics::RenderWindow,
        provider: &mut dyn Input_Provider,
    ) {
        let events = provider.poll_events(window);
        self.read_events_to_actions(&events);
    }

    #[cfg(feature = "use-sfml")]
    fn read_events_to_actions(&mut self, events: &[sfml::window::Event]) {
        use sfml::window::Event;

        let bindings = &self.bindings;
        self.actions.actions.clear();

        let handle_actions = |actions: &mut Action_List,
                              kind: Action_Kind,
                              names: &Vec<_>,
                              mappings: &Action_Callbacks| {
            for name in names.iter() {
                if let Some(callbacks) = mappings.get_callbacks_for_action(*name, kind) {
                    for callback in callbacks.iter() {
                        callback(actions);
                    }
                }
            }
        };

        for &event in events.iter() {
            match event {
                Event::Closed { .. } => self.actions.actions.push(Action::Quit),
                Event::KeyPressed { code, .. } => {
                    if let Some(action_names) = bindings.get_key_action(code) {
                        handle_actions(
                            &mut self.actions,
                            Action_Kind::Pressed,
                            action_names,
                            &self.action_callbacks,
                        );
                    }
                }
                Event::KeyReleased { code, .. } => {
                    if let Some(action_names) = bindings.get_key_action(code) {
                        handle_actions(
                            &mut self.actions,
                            Action_Kind::Released,
                            action_names,
                            &self.action_callbacks,
                        );
                    }
                }
                Event::JoystickButtonPressed { joystickid, button } => {
                    if let Some(action_names) = bindings.get_joystick_action(joystickid, button) {
                        handle_actions(
                            &mut self.actions,
                            Action_Kind::Pressed,
                            action_names,
                            &self.action_callbacks,
                        );
                    }
                }
                Event::JoystickButtonReleased { joystickid, button } => {
                    if let Some(action_names) = bindings.get_joystick_action(joystickid, button) {
                        handle_actions(
                            &mut self.actions,
                            Action_Kind::Released,
                            action_names,
                            &self.action_callbacks,
                        );
                    }
                }
                Event::MouseButtonPressed { button, .. } => {
                    if let Some(action_names) = bindings.get_mouse_action(button) {
                        handle_actions(
                            &mut self.actions,
                            Action_Kind::Pressed,
                            action_names,
                            &self.action_callbacks,
                        );
                    }
                }
                Event::MouseButtonReleased { button, .. } => {
                    if let Some(action_names) = bindings.get_mouse_action(button) {
                        handle_actions(
                            &mut self.actions,
                            Action_Kind::Released,
                            action_names,
                            &self.action_callbacks,
                        );
                    }
                }
                _ => (),
            }
        }
    }
}

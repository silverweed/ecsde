use super::axes;
use super::bindings::{Axis_Emulation_Type, Input_Bindings};
use super::core_actions::Core_Action;
use super::provider::{Input_Provider, Input_Provider_Input, Input_Provider_Output};
use crate::core::common::stringid::String_Id;
use crate::core::env::Env_Info;
use std::collections::HashSet;

pub struct Default_Input_Provider {}

/// The default input provider just gets all events from the window
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

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub enum Action_Kind {
    Pressed,
    Released,
}

pub type Game_Action = (String_Id, Action_Kind);

pub struct Input_System {
    // Input configuration
    bindings: Input_Bindings,

    // Output values
    core_actions: Vec<Core_Action>,
    game_actions: Vec<Game_Action>,
    axes: axes::Virtual_Axes,
}

impl Input_System {
    pub fn new(env: &Env_Info) -> Input_System {
        let bindings = super::create_bindings(env);
        let axes = axes::Virtual_Axes::with_axes(bindings.get_all_virtual_axes_names());
        Input_System {
            bindings,
            core_actions: vec![],
            game_actions: vec![],
            axes,
        }
    }

    pub fn get_game_actions(&self) -> &[Game_Action] {
        &self.game_actions
    }

    pub fn get_core_actions(&self) -> &[Core_Action] {
        &self.core_actions
    }

    pub fn get_axes(&self) -> &axes::Virtual_Axes {
        &self.axes
    }

    #[cfg(feature = "use-sfml")]
    pub fn update(
        &mut self,
        window: &mut sfml::graphics::RenderWindow,
        provider: &mut dyn Input_Provider,
    ) {
        let events = provider.poll_events(window);
        self.update_real_axes(); // Note: these axes values may be later overwritten by actions
        self.read_events_to_actions(&events);
    }

    #[cfg(feature = "use-sfml")]
    fn read_events_to_actions(&mut self, events: &[sfml::window::Event]) {
        use sfml::window::Event;

        let bindings = &self.bindings;
        self.core_actions.clear();
        self.game_actions.clear();

        fn axis_value_emu(emu: &Axis_Emulation_Type) -> f32 {
            match emu {
                Axis_Emulation_Type::Min => -1.0,
                Axis_Emulation_Type::Max => 1.0,
            }
        }

        let handle_actions =
            |actions: &mut Vec<Game_Action>, kind: Action_Kind, names: &[String_Id]| {
                for name in names.iter() {
                    actions.push((*name, kind));
                }
            };

        let handle_axis_pressed =
            |axes: &mut axes::Virtual_Axes, names: &[(String_Id, Axis_Emulation_Type)]| {
                for (axis_name, emu_kind) in names.iter() {
                    axes.set_emulated_value(*axis_name, axis_value_emu(emu_kind));
                }
            };

        let handle_axis_released =
            |axes: &mut axes::Virtual_Axes, names: &[(String_Id, Axis_Emulation_Type)]| {
                for (axis_name, _) in names.iter() {
                    axes.reset_emulated_value(*axis_name);
                }
            };

        for &event in events.iter() {
            match event {
                Event::Closed { .. } => self.core_actions.push(Core_Action::Quit),
                Event::Resized { width, height } => {
                    self.core_actions.push(Core_Action::Resize(width, height))
                }
                Event::KeyPressed { code, .. } => {
                    if let Some(names) = bindings.get_key_actions(code) {
                        handle_actions(&mut self.game_actions, Action_Kind::Pressed, names);
                    }
                    if let Some(names) = bindings.get_key_emulated_axes(code) {
                        handle_axis_pressed(&mut self.axes, names);
                    }
                }
                Event::KeyReleased { code, .. } => {
                    if let Some(names) = bindings.get_key_actions(code) {
                        handle_actions(&mut self.game_actions, Action_Kind::Released, names);
                    }
                    if let Some(names) = bindings.get_key_emulated_axes(code) {
                        handle_axis_released(&mut self.axes, names);
                    }
                }
                Event::JoystickButtonPressed { joystickid, button } => {
                    if let Some(names) = bindings.get_joystick_actions(joystickid, button) {
                        handle_actions(&mut self.game_actions, Action_Kind::Pressed, names);
                    }
                    if let Some(names) = bindings.get_joystick_emulated_axes(joystickid, button) {
                        handle_axis_pressed(&mut self.axes, names);
                    }
                }
                Event::JoystickButtonReleased { joystickid, button } => {
                    if let Some(names) = bindings.get_joystick_actions(joystickid, button) {
                        handle_actions(&mut self.game_actions, Action_Kind::Released, names);
                    }
                    if let Some(names) = bindings.get_joystick_emulated_axes(joystickid, button) {
                        handle_axis_released(&mut self.axes, names);
                    }
                }
                Event::MouseButtonPressed { button, .. } => {
                    if let Some(names) = bindings.get_mouse_actions(button) {
                        handle_actions(&mut self.game_actions, Action_Kind::Pressed, names);
                    }
                    if let Some(names) = bindings.get_mouse_emulated_axes(button) {
                        handle_axis_pressed(&mut self.axes, names);
                    }
                }
                Event::MouseButtonReleased { button, .. } => {
                    if let Some(names) = bindings.get_mouse_actions(button) {
                        handle_actions(&mut self.game_actions, Action_Kind::Released, names);
                    }
                    if let Some(names) = bindings.get_mouse_emulated_axes(button) {
                        handle_axis_released(&mut self.axes, names);
                    }
                }
                _ => (),
            }
        }
    }

    fn update_real_axes(&mut self) {
        let emulated: HashSet<String_Id> = self.axes.value_comes_from_emulation.clone();
        for (name, val) in self.axes.values.iter_mut() {
            if !emulated.contains(name) {
                *val = 0.0;
            }
        }
    }
}

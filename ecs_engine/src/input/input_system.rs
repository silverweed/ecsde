use super::axes;
use super::bindings::joystick;
use super::bindings::{Axis_Emulation_Type, Input_Bindings};
use super::core_actions::Core_Action;
use super::joystick_mgr::{Joystick_Manager, Real_Axes_Values};
use super::provider::{Input_Provider, Input_Provider_Input};
use crate::cfg;
use crate::common::stringid::String_Id;
use crate::common::Maybe_Error;
use crate::core::env::Env_Info;
use std::convert::TryInto;

#[cfg(feature = "use-sfml")]
use sfml::window::Event;

#[cfg(feature = "use-sfml")]
pub type Input_Raw_Event = sfml::window::Event;

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub enum Action_Kind {
    Pressed,
    Released,
}

pub type Game_Action = (String_Id, Action_Kind);

pub struct Input_System {
    joystick_mgr: Joystick_Manager,

    // Input configuration
    bindings: Input_Bindings,

    // Output values
    core_actions: Vec<Core_Action>,
    game_actions: Vec<Game_Action>,
    axes: axes::Virtual_Axes,
    /// Contains the raw window events that are suitable for becoming game actions.
    raw_events: Vec<Input_Raw_Event>,
}

impl Input_System {
    pub fn new(env: &Env_Info) -> Input_System {
        let bindings = super::create_bindings(env);
        let axes = axes::Virtual_Axes::with_axes(bindings.get_all_virtual_axes_names());
        Input_System {
            joystick_mgr: Joystick_Manager::new(),
            bindings,
            core_actions: vec![],
            game_actions: vec![],
            axes,
            raw_events: vec![],
        }
    }

    pub fn init(&mut self) -> Maybe_Error {
        self.joystick_mgr.init()
    }

    pub fn extract_game_actions(&mut self) -> Vec<Game_Action> {
        let mut v = vec![];
        std::mem::swap(&mut self.game_actions, &mut v);
        v
    }

    pub fn extract_core_actions(&mut self) -> Vec<Core_Action> {
        let mut v = vec![];
        std::mem::swap(&mut self.core_actions, &mut v);
        v
    }

    pub fn get_virtual_axes(&self) -> &axes::Virtual_Axes {
        &self.axes
    }

    pub fn get_real_axes(&self, joystick: joystick::Joystick) -> Option<&Real_Axes_Values> {
        self.joystick_mgr.values(joystick)
    }

    /// Returns ([all axes values for all joysticks], bitmask for connected joysticks)
    pub fn get_all_real_axes(&self) -> (&[Real_Axes_Values; joystick::JOY_COUNT as usize], u8) {
        self.joystick_mgr.all_values()
    }

    pub fn get_raw_events(&self) -> &[Input_Raw_Event] {
        &self.raw_events
    }

    pub fn get_bindings(&self) -> &Input_Bindings {
        &self.bindings
    }

    pub fn update(
        &mut self,
        window: &mut Input_Provider_Input,
        provider: &mut dyn Input_Provider,
        cfg: &cfg::Config,
    ) {
        joystick::update_joysticks();
        provider.update(window, Some(&self.joystick_mgr), cfg);

        self.joystick_mgr.update_from_input_provider(provider);
        self.update_real_axes(); // Note: these axes values may be later overwritten by actions

        let events = provider.get_events();
        self.read_events_to_actions(events);
    }

    fn read_events_to_actions(&mut self, events: &[Input_Raw_Event]) {
        let bindings = &self.bindings;
        self.core_actions.clear();
        self.game_actions.clear();

        let handle_actions =
            |actions: &mut Vec<Game_Action>, kind: Action_Kind, names: &[String_Id]| {
                for name in names.iter() {
                    actions.push((*name, kind));
                }
            };

        let handle_axis_pressed =
            |axes: &mut axes::Virtual_Axes, names: &[(String_Id, Axis_Emulation_Type)]| {
                for (axis_name, emu_kind) in names.iter() {
                    axes.set_emulated_value(*axis_name, *emu_kind);
                }
            };

        let handle_axis_released =
            |axes: &mut axes::Virtual_Axes, names: &[(String_Id, Axis_Emulation_Type)]| {
                for (axis_name, emu_kind) in names.iter() {
                    axes.reset_emulated_value(*axis_name, *emu_kind);
                }
            };

        self.raw_events.clear();

        for &event in events.iter() {
            self.raw_events.push(event);
            let prev_core_actions_len = self.core_actions.len();

            #[cfg(feature = "use-sfml")]
            match event {
                // Core Actions
                Event::Closed { .. } => self.core_actions.push(Core_Action::Quit),
                Event::Resized { width, height } => {
                    self.core_actions.push(Core_Action::Resize(width, height))
                }
                // Game Actions
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
                    if let Some(names) =
                        bindings.get_joystick_actions(joystickid, button, &self.joystick_mgr)
                    {
                        handle_actions(&mut self.game_actions, Action_Kind::Pressed, names);
                    }
                    if let Some(names) =
                        bindings.get_joystick_emulated_axes(joystickid, button, &self.joystick_mgr)
                    {
                        handle_axis_pressed(&mut self.axes, names);
                    }
                }
                Event::JoystickButtonReleased { joystickid, button } => {
                    if let Some(names) =
                        bindings.get_joystick_actions(joystickid, button, &self.joystick_mgr)
                    {
                        handle_actions(&mut self.game_actions, Action_Kind::Released, names);
                    }
                    if let Some(names) =
                        bindings.get_joystick_emulated_axes(joystickid, button, &self.joystick_mgr)
                    {
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
                Event::MouseWheelScrolled { delta, .. } => {
                    if let Some(names) = bindings.get_mouse_wheel_actions(delta > 0.) {
                        // Note: MouseWheel actions always count as 'Pressed'.
                        handle_actions(&mut self.game_actions, Action_Kind::Pressed, names);
                    }
                    if let Some(names) = bindings.get_mouse_wheel_emulated_axes(delta > 0.) {
                        handle_axis_pressed(&mut self.axes, names);
                    }
                }
                Event::JoystickConnected { joystickid } => {
                    self.joystick_mgr.register(joystickid);
                }
                Event::JoystickDisconnected { joystickid } => {
                    self.joystick_mgr.unregister(joystickid);
                }
                _ => {
                    // We're not interested in this event.
                    self.raw_events.pop();
                }
            }

            if self.core_actions.len() > prev_core_actions_len {
                // The event was a core action: don't save it in raw_events
                self.raw_events.pop();
            }
        }
    }

    // @Speed: we can probably do better than all these map reads
    fn update_real_axes(&mut self) {
        let axes = &mut self.axes;
        for (name, val) in axes.values.iter_mut() {
            if let Some((min, max)) = axes.value_comes_from_emulation.get(name) {
                if *min || *max {
                    continue;
                }
            }
            *val = 0.0;
        }

        let bindings = &self.bindings;

        let (all_real_axes, joy_mask) = self.joystick_mgr.all_values();

        for (joy_id, real_axes) in all_real_axes.iter().enumerate() {
            if (joy_mask & (1 << joy_id)) == 0 {
                continue;
            }

            for i in 0u8..joystick::Joystick_Axis::_Count as u8 {
                let axis = i.try_into().unwrap_or_else(|err| {
                    panic!("Failed to convert {} to a valid Joystick_Axis: {}", i, err)
                });

                for virtual_axis_name in bindings.get_virtual_axes_from_real_axis(axis) {
                    if let Some((min, max)) = axes.value_comes_from_emulation.get(virtual_axis_name)
                    {
                        if *min || *max {
                            continue;
                        }
                    }
                    let cur_value = axes.values.get_mut(&virtual_axis_name).unwrap();
                    let new_value = real_axes[i as usize];

                    // It may be the case that multiple real axes map to the same virtual axis.
                    // For now, we keep the value that has the maximum absolute value.
                    if new_value.abs() > cur_value.abs() {
                        *cur_value = new_value;
                    }
                }
            }
        }
    }
}

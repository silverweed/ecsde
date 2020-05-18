use super::axes;
use super::bindings::joystick;
use super::bindings::mouse::{self, Mouse_State};
use super::bindings::{Axis_Emulation_Type, Input_Bindings};
use super::core_actions::Core_Action;
use super::joystick_state::{self, Joystick_State};
use super::provider::{Input_Provider, Input_Provider_Input};
use crate::cfg;
use crate::common::stringid::String_Id;
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

pub struct Input_State {
    pub joy_state: Joystick_State,
    pub mouse_state: Mouse_State,

    // Input configuration
    pub bindings: Input_Bindings,

    // Output values
    pub core_actions: Vec<Core_Action>,
    pub game_actions: Vec<Game_Action>,
    pub axes: axes::Virtual_Axes,
    /// Contains the raw window events that are suitable for becoming game actions.
    pub raw_events: Vec<Input_Raw_Event>,
}

pub fn create_input_state(env: &Env_Info) -> Input_State {
    let bindings = super::create_bindings(env);
    let axes = axes::Virtual_Axes::with_axes(&bindings.axis_bindings.axes_names);
    Input_State {
        joy_state: Joystick_State::default(),
        mouse_state: Mouse_State::default(),
        bindings,
        core_actions: vec![],
        game_actions: vec![],
        axes,
        raw_events: vec![],
    }
}

pub fn update_input(
    input_state: &mut Input_State,
    window: &mut Input_Provider_Input,
    provider: &mut dyn Input_Provider,
    cfg: &cfg::Config,
    process_game_actions: bool,
) {
    joystick::update_joysticks();
    mouse::update_mouse_state(&mut input_state.mouse_state);
    provider.update(window, Some(&input_state.joy_state), cfg);

    provider.get_axes(&mut input_state.joy_state.values);
    update_real_axes(input_state); // Note: these axes values may be later overwritten by actions

    let events = provider.get_events();
    read_events_to_actions(input_state, events, process_game_actions);
}

fn handle_actions(actions: &mut Vec<Game_Action>, kind: Action_Kind, names: &[String_Id]) {
    for name in names.iter() {
        actions.push((*name, kind));
    }
}

fn handle_axis_pressed(axes: &mut axes::Virtual_Axes, names: &[(String_Id, Axis_Emulation_Type)]) {
    for (axis_name, emu_kind) in names.iter() {
        axes.set_emulated_value(*axis_name, *emu_kind);
    }
}

fn handle_axis_released(axes: &mut axes::Virtual_Axes, names: &[(String_Id, Axis_Emulation_Type)]) {
    for (axis_name, emu_kind) in names.iter() {
        axes.reset_emulated_value(*axis_name, *emu_kind);
    }
}

fn read_events_to_actions(
    state: &mut Input_State,
    events: &[Input_Raw_Event],
    process_game_actions: bool,
) {
    state.core_actions.clear();
    state.game_actions.clear();
    state.raw_events.clear();

    let process_event_func = if process_game_actions {
        process_event_core_and_game_actions
    } else {
        process_event_core_actions
    };

    for &event in events.iter() {
        state.raw_events.push(event);
        process_event_func(state, event);
    }
}

fn process_event_core_and_game_actions(state: &mut Input_State, event: Input_Raw_Event) -> bool {
    if process_event_core_actions(state, event) {
        return true;
    }
    process_event_game_actions(state, event)
}

#[cfg(feature = "use-sfml")]
fn process_event_core_actions(state: &mut Input_State, event: Input_Raw_Event) -> bool {
    match event {
        Event::Closed { .. } => state.core_actions.push(Core_Action::Quit),
        Event::Resized { width, height } => {
            state.core_actions.push(Core_Action::Resize(width, height))
        }
        _ => {
            return false;
        }
    }
    true
}

#[cfg(feature = "use-sfml")]
fn process_event_game_actions(state: &mut Input_State, event: Input_Raw_Event) -> bool {
    let bindings = &state.bindings;
    match event {
        // Game Actions
        Event::KeyPressed { code, .. } => {
            if let Some(names) = bindings.get_key_actions(code) {
                handle_actions(&mut state.game_actions, Action_Kind::Pressed, names);
            }
            if let Some(names) = bindings.get_key_emulated_axes(code) {
                handle_axis_pressed(&mut state.axes, names);
            }
        }
        Event::KeyReleased { code, .. } => {
            if let Some(names) = bindings.get_key_actions(code) {
                handle_actions(&mut state.game_actions, Action_Kind::Released, names);
            }
            if let Some(names) = bindings.get_key_emulated_axes(code) {
                handle_axis_released(&mut state.axes, names);
            }
        }
        Event::JoystickButtonPressed { joystickid, button } => {
            if let Some(names) = bindings.get_joystick_actions(joystickid, button, &state.joy_state)
            {
                handle_actions(&mut state.game_actions, Action_Kind::Pressed, names);
            }
            if let Some(names) =
                bindings.get_joystick_emulated_axes(joystickid, button, &state.joy_state)
            {
                handle_axis_pressed(&mut state.axes, names);
            }
        }
        Event::JoystickButtonReleased { joystickid, button } => {
            if let Some(names) = bindings.get_joystick_actions(joystickid, button, &state.joy_state)
            {
                handle_actions(&mut state.game_actions, Action_Kind::Released, names);
            }
            if let Some(names) =
                bindings.get_joystick_emulated_axes(joystickid, button, &state.joy_state)
            {
                handle_axis_released(&mut state.axes, names);
            }
        }
        Event::MouseButtonPressed { button, .. } => {
            if let Some(names) = bindings.get_mouse_actions(button) {
                handle_actions(&mut state.game_actions, Action_Kind::Pressed, names);
            }
            if let Some(names) = bindings.get_mouse_emulated_axes(button) {
                handle_axis_pressed(&mut state.axes, names);
            }
        }
        Event::MouseButtonReleased { button, .. } => {
            if let Some(names) = bindings.get_mouse_actions(button) {
                handle_actions(&mut state.game_actions, Action_Kind::Released, names);
            }
            if let Some(names) = bindings.get_mouse_emulated_axes(button) {
                handle_axis_released(&mut state.axes, names);
            }
        }
        Event::MouseWheelScrolled { delta, .. } => {
            if let Some(names) = bindings.get_mouse_wheel_actions(delta > 0.) {
                // Note: MouseWheel actions always count as 'Pressed'.
                handle_actions(&mut state.game_actions, Action_Kind::Pressed, names);
            }
            if let Some(names) = bindings.get_mouse_wheel_emulated_axes(delta > 0.) {
                handle_axis_pressed(&mut state.axes, names);
            }
        }
        Event::JoystickConnected { joystickid } => {
            joystick_state::register_joystick(&mut state.joy_state, joystickid);
        }
        Event::JoystickDisconnected { joystickid } => {
            joystick_state::unregister_joystick(&mut state.joy_state, joystickid);
        }
        _ => {
            return false;
        }
    }
    true
}

// @Speed: we can probably do better than all these map reads
fn update_real_axes(state: &mut Input_State) {
    let axes = &mut state.axes;
    for (name, val) in axes.values.iter_mut() {
        if let Some((min, max)) = axes.value_comes_from_emulation.get(name) {
            if *min || *max {
                continue;
            }
        }
        *val = 0.0;
    }

    let bindings = &state.bindings;

    let (all_real_axes, joy_mask) = joystick_state::all_joysticks_values(&state.joy_state);

    for (joy_id, real_axes) in all_real_axes.iter().enumerate() {
        if (joy_mask & (1 << joy_id)) == 0 {
            continue;
        }

        for i in 0u8..joystick::Joystick_Axis::_Count as u8 {
            let axis = i.try_into().unwrap_or_else(|err| {
                panic!("Failed to convert {} to a valid Joystick_Axis: {}", i, err)
            });

            for virtual_axis_name in bindings.get_virtual_axes_from_real_axis(axis) {
                if let Some((min, max)) = axes.value_comes_from_emulation.get(virtual_axis_name) {
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

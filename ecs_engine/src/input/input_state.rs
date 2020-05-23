use super::axes;
use super::bindings::joystick;
use super::bindings::mouse::{self, Mouse_State};
use super::bindings::{Axis_Emulation_Type, Input_Bindings};
use super::core_actions::Core_Action;
use super::joystick_state::{self, Joystick_State};
use crate::common::stringid::String_Id;
use crate::core::env::Env_Info;
use crate::gfx::window::{self, Event, Window_Handle};
use std::convert::TryInto;

#[cfg(feature = "use-sfml")]
pub type Input_Raw_Event = sfml::window::Event;

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub enum Action_Kind {
    Pressed,
    Released,
}

pub type Game_Action = (String_Id, Action_Kind);

#[derive(Clone, Default)]
pub struct Input_Raw_State {
    pub joy_state: Joystick_State,
    pub mouse_state: Mouse_State,
    pub events: Vec<Input_Raw_Event>,
}

pub struct Processed_Input {
    pub core_actions: Vec<Core_Action>,
    pub game_actions: Vec<Game_Action>,
    pub virtual_axes: axes::Virtual_Axes,
}

pub struct Input_State {
    pub bindings: Input_Bindings,
    pub raw_state: Input_Raw_State,
    pub processed: Processed_Input,
}

pub fn create_input_state(env: &Env_Info) -> Input_State {
    let bindings = super::create_bindings(env);
    let virtual_axes = axes::Virtual_Axes::with_axes(&bindings.axis_bindings.axes_names);
    Input_State {
        raw_state: Input_Raw_State {
            joy_state: Joystick_State::default(),
            mouse_state: Mouse_State::default(),
            events: vec![],
        },
        bindings,
        processed: Processed_Input {
            core_actions: vec![],
            game_actions: vec![],
            virtual_axes,
        },
    }
}

pub fn update_raw_input<W: AsMut<Window_Handle>>(window: &mut W, raw_state: &mut Input_Raw_State) {
    let window = window.as_mut();

    joystick::update_joysticks();
    mouse::update_mouse_state(&mut raw_state.mouse_state);

    raw_state.events.clear();

    while let Some(evt) = window::poll_event(window) {
        match evt {
            Event::JoystickConnected { joystickid } => {
                joystick_state::register_joystick(&mut raw_state.joy_state, joystickid);
            }
            Event::JoystickDisconnected { joystickid } => {
                joystick_state::unregister_joystick(&mut raw_state.joy_state, joystickid);
            }
            _ => raw_state.events.push(evt),
        }
    }

    for joy_id in 0..joystick::JOY_COUNT {
        if let Some(joy) = &raw_state.joy_state.joysticks[joy_id as usize] {
            for i in 0u8..joystick::Joystick_Axis::_Count as u8 {
                let axis = unsafe { std::mem::transmute(i) };
                raw_state.joy_state.values[joy_id as usize][i as usize] =
                    joystick::get_joy_axis_value(*joy, axis);
            }
        }
    }
}

pub fn process_raw_input(
    raw_state: &Input_Raw_State,
    bindings: &Input_Bindings,
    processed: &mut Processed_Input,
    process_game_actions: bool,
) {
    update_virtual_axes_from_real_axes(raw_state, bindings, processed); // Note: these axes values may be later overwritten by actions
    read_events_to_actions(raw_state, bindings, processed, process_game_actions);
}

fn read_events_to_actions(
    raw_state: &Input_Raw_State,
    bindings: &Input_Bindings,
    processed: &mut Processed_Input,
    process_game_actions: bool,
) {
    processed.core_actions.clear();
    processed.game_actions.clear();

    let process_event_func = if process_game_actions {
        process_event_core_and_game_actions
    } else {
        process_event_core_actions
    };

    for &event in raw_state.events.iter() {
        process_event_func(event, raw_state, bindings, processed);
    }
}

fn process_event_core_and_game_actions(
    event: Input_Raw_Event,
    raw_state: &Input_Raw_State,
    bindings: &Input_Bindings,
    processed: &mut Processed_Input,
) -> bool {
    if process_event_core_actions(event, raw_state, bindings, processed) {
        return true;
    }
    process_event_game_actions(event, raw_state, bindings, processed)
}

#[cfg(feature = "use-sfml")]
fn process_event_core_actions(
    event: Input_Raw_Event,
    _raw_state: &Input_Raw_State,
    _bindings: &Input_Bindings,
    processed: &mut Processed_Input,
) -> bool {
    match event {
        Event::Closed { .. } => processed.core_actions.push(Core_Action::Quit),
        Event::Resized { width, height } => processed
            .core_actions
            .push(Core_Action::Resize(width, height)),
        _ => {
            return false;
        }
    }
    true
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

#[cfg(feature = "use-sfml")]
fn process_event_game_actions(
    event: Input_Raw_Event,
    raw_state: &Input_Raw_State,
    bindings: &Input_Bindings,
    processed: &mut Processed_Input,
) -> bool {
    match event {
        // Game Actions
        Event::KeyPressed { code, .. } => {
            if let Some(names) = bindings.get_key_actions(code) {
                handle_actions(&mut processed.game_actions, Action_Kind::Pressed, names);
            }
            if let Some(names) = bindings.get_key_emulated_axes(code) {
                handle_axis_pressed(&mut processed.virtual_axes, names);
            }
        }
        Event::KeyReleased { code, .. } => {
            if let Some(names) = bindings.get_key_actions(code) {
                handle_actions(&mut processed.game_actions, Action_Kind::Released, names);
            }
            if let Some(names) = bindings.get_key_emulated_axes(code) {
                handle_axis_released(&mut processed.virtual_axes, names);
            }
        }
        Event::JoystickButtonPressed { joystickid, button } => {
            if let Some(names) =
                bindings.get_joystick_actions(joystickid, button, &raw_state.joy_state)
            {
                handle_actions(&mut processed.game_actions, Action_Kind::Pressed, names);
            }
            if let Some(names) =
                bindings.get_joystick_emulated_axes(joystickid, button, &raw_state.joy_state)
            {
                handle_axis_pressed(&mut processed.virtual_axes, names);
            }
        }
        Event::JoystickButtonReleased { joystickid, button } => {
            if let Some(names) =
                bindings.get_joystick_actions(joystickid, button, &raw_state.joy_state)
            {
                handle_actions(&mut processed.game_actions, Action_Kind::Released, names);
            }
            if let Some(names) =
                bindings.get_joystick_emulated_axes(joystickid, button, &raw_state.joy_state)
            {
                handle_axis_released(&mut processed.virtual_axes, names);
            }
        }
        Event::MouseButtonPressed { button, .. } => {
            if let Some(names) = bindings.get_mouse_actions(button) {
                handle_actions(&mut processed.game_actions, Action_Kind::Pressed, names);
            }
            if let Some(names) = bindings.get_mouse_emulated_axes(button) {
                handle_axis_pressed(&mut processed.virtual_axes, names);
            }
        }
        Event::MouseButtonReleased { button, .. } => {
            if let Some(names) = bindings.get_mouse_actions(button) {
                handle_actions(&mut processed.game_actions, Action_Kind::Released, names);
            }
            if let Some(names) = bindings.get_mouse_emulated_axes(button) {
                handle_axis_released(&mut processed.virtual_axes, names);
            }
        }
        Event::MouseWheelScrolled { delta, .. } => {
            if let Some(names) = bindings.get_mouse_wheel_actions(delta > 0.) {
                // Note: MouseWheel actions always count as 'Pressed'.
                handle_actions(&mut processed.game_actions, Action_Kind::Pressed, names);
            }
            if let Some(names) = bindings.get_mouse_wheel_emulated_axes(delta > 0.) {
                handle_axis_pressed(&mut processed.virtual_axes, names);
            }
        }
        _ => {
            return false;
        }
    }
    true
}

// @Speed: we can probably do better than all these map reads
fn update_virtual_axes_from_real_axes(
    raw_state: &Input_Raw_State,
    bindings: &Input_Bindings,
    processed: &mut Processed_Input,
) {
    // Clear all virtual axes unless they come from emulation (i.e. a key linked to an axis was pressed)
    let virtual_axes = &mut processed.virtual_axes;
    for (name, val) in virtual_axes.values.iter_mut() {
        if let Some((min, max)) = virtual_axes.value_comes_from_emulation.get(name) {
            if *min || *max {
                continue;
            }
        }
        *val = 0.0;
    }

    let (all_real_axes, joy_mask) = joystick_state::all_joysticks_values(&raw_state.joy_state);

    // Update virtual virtual_axes from real virtual_axes values
    for (joy_id, real_axes) in all_real_axes.iter().enumerate() {
        if (joy_mask & (1 << joy_id)) == 0 {
            continue;
        }

        for i in 0u8..joystick::Joystick_Axis::_Count as u8 {
            let axis = i.try_into().unwrap_or_else(|err| {
                panic!("Failed to convert {} to a valid Joystick_Axis: {}", i, err)
            });

            for virtual_axis_name in bindings.get_virtual_axes_from_real_axis(axis) {
                if let Some((min, max)) = virtual_axes
                    .value_comes_from_emulation
                    .get(virtual_axis_name)
                {
                    if *min || *max {
                        continue;
                    }
                }
                let cur_value = virtual_axes.values.get_mut(&virtual_axis_name).unwrap();
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

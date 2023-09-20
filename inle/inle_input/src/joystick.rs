use crate::events::Input_Raw_Event;
use inle_core::env::Env_Info;
use inle_win::window::Window_Handle;
use std::convert::TryFrom;
use std::fmt;

#[cfg(feature = "win-glfw")]
mod glfw;

#[cfg(feature = "win-glfw")]
use self::glfw as backend;

pub const JOY_COUNT: u8 = 8;

pub type Joystick_Mask = u8;
pub type Joystick_Id = u32;
type Button_Id = u32;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Joystick_Type {
    XBox360,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Joystick {
    pub id: Joystick_Id,
    pub joy_type: Joystick_Type,
}

// Don't change the order of these! @Volatile with the mappings below.
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum Joystick_Button {
    /// Triangle
    Face_Top,
    /// Circle
    Face_Right,
    /// Square
    Face_Bottom,
    /// Cross
    Face_Left,
    /// Select
    Special_Left,
    /// Start
    Special_Right,
    /// XBox Button
    Special_Middle,
    /// L3
    Stick_Left,
    /// R3
    Stick_Right,
    /// L1
    Shoulder_Left,
    /// R1
    Shoulder_Right,
    Dpad_Top,
    Dpad_Right,
    Dpad_Bottom,
    Dpad_Left,
    /// L2
    Trigger_Left,
    /// R2
    Trigger_Right,
    _Count,
}

impl TryFrom<u8> for Joystick_Button {
    type Error = String;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        if v >= Joystick_Button::_Count as u8 {
            return Err(format!("Invalid Joystick_Button: {}", v));
        }
        Ok(unsafe { std::mem::transmute(v) })
    }
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum Joystick_Axis {
    Stick_Left_H,
    Stick_Left_V,
    Stick_Right_H,
    Stick_Right_V,
    Trigger_Left,
    Trigger_Right,
    Dpad_H,
    Dpad_V,
    _Count,
}

impl TryFrom<u8> for Joystick_Axis {
    type Error = String;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        if v >= Joystick_Axis::_Count as u8 {
            return Err(format!("Invalid Joystick_Axis: {}", v));
        }
        Ok(unsafe { std::mem::transmute(v) })
    }
}

pub fn string_to_joy_btn(s: &str) -> Option<Joystick_Button> {
    match s {
        "Face_Top" | "Triangle" | "XBox_Y" => Some(Joystick_Button::Face_Top),
        "Face_Right" | "Circle" | "XBox_B" => Some(Joystick_Button::Face_Right),
        "Face_Bottom" | "Cross" | "XBox_A" => Some(Joystick_Button::Face_Bottom),
        "Face_Left" | "Square" | "XBox_X" => Some(Joystick_Button::Face_Left),
        "Special_Left" | "Select" => Some(Joystick_Button::Special_Left),
        "Special_Right" | "Start" => Some(Joystick_Button::Special_Right),
        "Special_Middle" => Some(Joystick_Button::Special_Middle),
        "Stick_Left" | "L3" => Some(Joystick_Button::Stick_Left),
        "Stick_Right" | "R3" => Some(Joystick_Button::Stick_Right),
        "Shoulder_Left" | "L1" | "LB" => Some(Joystick_Button::Shoulder_Left),
        "Shoulder_Right" | "R1" | "RB" => Some(Joystick_Button::Shoulder_Right),
        "Dpad_Top" => Some(Joystick_Button::Dpad_Top),
        "Dpad_Right" => Some(Joystick_Button::Dpad_Right),
        "Dpad_Bottom" => Some(Joystick_Button::Dpad_Bottom),
        "Dpad_Left" => Some(Joystick_Button::Dpad_Left),
        "Trigger_Left" | "L2" | "LT" => Some(Joystick_Button::Trigger_Left),
        "Trigger_Right" | "R2" | "RT" => Some(Joystick_Button::Trigger_Right),
        _ => None,
    }
}

pub fn string_to_joy_axis(s: &str) -> Option<Joystick_Axis> {
    match s {
        "Stick_Left_H" => Some(Joystick_Axis::Stick_Left_H),
        "Stick_Left_V" => Some(Joystick_Axis::Stick_Left_V),
        "Stick_Right_H" => Some(Joystick_Axis::Stick_Right_H),
        "Stick_Right_V" => Some(Joystick_Axis::Stick_Right_V),
        "Trigger_Left" | "L2" | "LT" => Some(Joystick_Axis::Trigger_Left),
        "Trigger_Right" | "R2" | "RT" => Some(Joystick_Axis::Trigger_Right),
        "Dpad_H" => Some(Joystick_Axis::Dpad_H),
        "Dpad_V" => Some(Joystick_Axis::Dpad_V),
        _ => None,
    }
}

#[inline]
pub fn is_joy_connected(state: &Joystick_State, id: Joystick_Id) -> bool {
    state.joysticks[id as usize].is_some()
}

#[inline]
pub fn get_joy_type(state: &Joystick_State, id: Joystick_Id) -> Option<Joystick_Type> {
    state.joysticks[id as usize].map(|j| j.joy_type)
}

#[inline]
pub fn get_connected_joysticks_mask(state: &Joystick_State) -> Joystick_Mask {
    let mut mask = 0u8;
    for i in 0..JOY_COUNT {
        mask |= (is_joy_connected(state, i.into()) as u8) << i;
    }
    mask
}

/// Returns the normalized value [-1, 1] of the given axis
pub fn get_joy_axis_value(state: &Joystick_State, joy_id: Joystick_Id, axis: Joystick_Axis) -> f32 {
    if state.joysticks[joy_id as usize].is_some() {
        state.axes[joy_id as usize][axis as usize]
    } else {
        0.0
    }
}

/// Returns the axes of all axes for a single joystick
pub fn get_joy_axes_values(
    joy_state: &Joystick_State,
    joy_id: Joystick_Id,
) -> Option<&Real_Axes_Values> {
    if joy_state.joysticks[joy_id as usize].is_some() {
        Some(&joy_state.axes[joy_id as usize])
    } else {
        None
    }
}

pub fn get_all_joysticks_axes_values(
    joy_state: &Joystick_State,
) -> (&[Real_Axes_Values; JOY_COUNT as usize], Joystick_Mask) {
    (&joy_state.axes, get_connected_joysticks_mask(joy_state))
}

pub type Real_Axes_Values = [f32; Joystick_Axis::_Count as usize];

#[derive(Copy, Clone, Default)]
pub struct Joystick_Button_Values {
    /// The bits in this mask are ordered as Joystick_Button, so they're now "raw"
    /// values but they are already reordered to match our enum's order, rather
    /// than the framework's.
    button_mask: u32,
}

const_assert!(
    std::mem::size_of::<u32>() * inle_common::WORD_SIZE >= Joystick_Button::_Count as usize
);

impl fmt::Debug for Joystick_Button_Values {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:b}", self.button_mask)
    }
}

impl Joystick_Button_Values {
    fn is_pressed(self, button: Joystick_Button) -> bool {
        (self.button_mask & (1 << button as u32)) != 0
    }

    fn set_pressed(&mut self, button: Joystick_Button, pressed: bool) {
        if pressed {
            self.button_mask |= 1 << button as u32;
        } else {
            self.button_mask &= !(1 << button as u32);
        }
    }
}

#[derive(Clone, Default, Debug)]
pub struct Joystick_State {
    pub joysticks: [Option<Joystick>; JOY_COUNT as usize],
    pub axes: [Real_Axes_Values; JOY_COUNT as usize],
    buttons: [Joystick_Button_Values; JOY_COUNT as usize],
    buttons_prev_state: [Joystick_Button_Values; JOY_COUNT as usize],
}

pub fn init_joysticks<T: AsRef<Window_Handle>>(
    window: &T,
    env: &Env_Info,
    joy_state: &mut Joystick_State,
) {
    let win = window.as_ref();

    backend::init_joysticks(win, env);

    update_joysticks();

    let mut joy_found = 0;
    for i in 0..(JOY_COUNT as u32) {
        let (connected, guid) = is_joy_connected_internal(win, i);
        if connected && register_joystick(joy_state, i, &guid) {
            joy_found += 1;
        }
    }

    linfo!("Found {} valid joysticks.", joy_found);
}

pub fn update_joystick_state(joy_state: &mut Joystick_State, events: &[Input_Raw_Event]) {
    for event in events {
        match event {
            Input_Raw_Event::Joy_Connected { id, guid } => {
                register_joystick(joy_state, *id, guid);
            }
            Input_Raw_Event::Joy_Disconnected { id, .. } => {
                unregister_joystick(joy_state, *id);
            }
            _ => {}
        }
    }
}

pub fn pre_update_joystick_state(
    window: &Window_Handle,
    joy_state: &mut Joystick_State,
    events_out: &mut Vec<Input_Raw_Event>,
) {
    update_joysticks();

    // Update axes and buttons
    for (joy_id, joy) in joy_state
        .joysticks
        .iter_mut()
        .filter_map(|j| j.as_ref())
        .enumerate()
    {
        for (axis_id, axis_val) in joy_state.axes[joy_id].iter_mut().enumerate() {
            *axis_val = get_joy_axis_value_internal(
                window,
                joy,
                Joystick_Axis::try_from(axis_id as u8).unwrap(),
            );
        }

        let btn_prev_values = joy_state.buttons[joy_id];
        joy_state.buttons_prev_state[joy_id] = btn_prev_values;

        let mut btn_values = Joystick_Button_Values::default();
        for btn_id in 0..Joystick_Button::_Count as usize {
            let joy_btn = Joystick_Button::try_from(btn_id as u8).unwrap();
            let pressed = is_joy_btn_pressed_internal(window, joy, joy_btn);
            btn_values.set_pressed(joy_btn, pressed);
        }
        joy_state.buttons[joy_id] = btn_values;

        // Create events for all buttons that flipped value
        let mut diff_values = btn_values.button_mask ^ btn_prev_values.button_mask;
        while diff_values != 0 {
            let first_diff_btn_id = diff_values.trailing_zeros();
            let evt = if btn_values.button_mask & (1 << first_diff_btn_id) != 0 {
                Input_Raw_Event::Joy_Button_Pressed {
                    joystick_id: joy_id as Joystick_Id,
                    button: Joystick_Button::try_from(first_diff_btn_id as u8).unwrap(),
                }
            } else {
                Input_Raw_Event::Joy_Button_Released {
                    joystick_id: joy_id as Joystick_Id,
                    button: Joystick_Button::try_from(first_diff_btn_id as u8).unwrap(),
                }
            };
            events_out.push(evt);

            diff_values &= !(1 << first_diff_btn_id);
        }
    }
}

fn register_joystick(
    joy_state: &mut Joystick_State,
    joystick_id: Joystick_Id,
    joystick_guid: &Option<Box<str>>,
) -> bool {
    ldebug!(
        "Registering joystick {} with guid {:?}",
        joystick_id,
        joystick_guid
    );

    match get_joy_type_internal(joystick_guid) {
        Ok(joy_type) => {
            ldebug!("Interpreting as type {:?}", joy_type);
            joy_state.joysticks[joystick_id as usize] = Some(Joystick {
                id: joystick_id,
                joy_type,
            });
            true
        }
        Err(msg) => {
            lwarn!("Failed to get type of joystick {}: {}", joystick_id, msg);
            false
        }
    }
}

fn unregister_joystick(joy_state: &mut Joystick_State, joystick_id: Joystick_Id) {
    ldebug!("Unregistering joystick {}", joystick_id);
    joy_state.joysticks[joystick_id as usize] = None;
}

fn get_joy_type_internal(_guid: &Option<Box<str>>) -> Result<Joystick_Type, &'static str> {
    // @Incomplete
    Ok(Joystick_Type::XBox360)
}

fn get_joy_axis_value_internal(
    window: &Window_Handle,
    joystick: &Joystick,
    axis: Joystick_Axis,
) -> f32 {
    match joystick.joy_type {
        Joystick_Type::XBox360 => get_joy_axis_value_xbox360(window, joystick.id, axis),
    }
}

fn is_joy_connected_internal(
    window: &Window_Handle,
    joy_id: Joystick_Id,
) -> (bool, Option<Box<str>>) {
    backend::is_joy_connected_internal(window, joy_id)
}

fn get_joy_axis_value_xbox360(
    window: &Window_Handle,
    joystick_id: Joystick_Id,
    axis: Joystick_Axis,
) -> f32 {
    backend::get_joy_axis_value_xbox360(window, joystick_id, axis)
}

fn is_joy_btn_pressed_internal(
    window: &Window_Handle,
    joy: &Joystick,
    btn: Joystick_Button,
) -> bool {
    match joy.joy_type {
        Joystick_Type::XBox360 => is_joy_btn_pressed_internal_xbox360(window, joy.id, btn),
    }
}

#[inline]
fn is_joy_btn_pressed_internal_xbox360(
    window: &Window_Handle,
    joy_id: Joystick_Id,
    btn: Joystick_Button,
) -> bool {
    backend::is_joy_btn_pressed_internal_xbox360(window, joy_id, btn)
}

/// Forces to update the joysticks' state
fn update_joysticks() {
    backend::update_joysticks();
}

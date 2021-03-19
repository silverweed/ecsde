use crate::events::Input_Raw_Event;
use inle_win::window::Window_Handle;
use std::convert::TryFrom;

#[cfg(feature = "win-sfml")]
mod sfml;

#[cfg(feature = "win-glfw")]
mod glfw;

#[cfg(feature = "win-sfml")]
use self::sfml as backend;

#[cfg(feature = "win-glfw")]
use self::glfw as backend;

pub const JOY_COUNT: u8 = 8;

pub type Joystick_Mask = u8;
pub type Joystick_Id = u32;
pub type Button_Id = u32;

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

pub fn get_joy_btn_id(joy_type: Joystick_Type, button: Joystick_Button) -> Option<Button_Id> {
    match joy_type {
        Joystick_Type::XBox360 => get_joy_btn_id_xbox360(button),
    }
}

pub fn get_joy_btn_from_id(joy_type: Joystick_Type, btn_id: Button_Id) -> Option<Joystick_Button> {
    match joy_type {
        Joystick_Type::XBox360 => get_joy_btn_from_id_xbox360(btn_id),
    }
}

/// Returns the normalized value [-1, 1] of the given axis
pub fn get_joy_axis_value(state: &Joystick_State, joy_id: Joystick_Id, axis: Joystick_Axis) -> f32 {
    if state.joysticks[joy_id as usize].is_some() {
        state.values[joy_id as usize][axis as usize]
    } else {
        0.0
    }
}

/// Returns the values of all axes for a single joystick
pub fn get_joy_axes_values(
    joy_state: &Joystick_State,
    joy_id: Joystick_Id,
) -> Option<&Real_Axes_Values> {
    if joy_state.joysticks[joy_id as usize].is_some() {
        Some(&joy_state.values[joy_id as usize])
    } else {
        None
    }
}

pub fn get_all_joysticks_axes_values(
    joy_state: &Joystick_State,
) -> (&[Real_Axes_Values; JOY_COUNT as usize], Joystick_Mask) {
    (&joy_state.values, get_connected_joysticks_mask(joy_state))
}

pub type Real_Axes_Values = [f32; Joystick_Axis::_Count as usize];

#[derive(Clone, Default, Debug)]
pub struct Joystick_State {
    pub joysticks: [Option<Joystick>; JOY_COUNT as usize],
    pub values: [Real_Axes_Values; JOY_COUNT as usize],
}

pub fn init_joysticks<T: AsRef<Window_Handle>>(window: &T, joy_state: &mut Joystick_State) {
    update_joysticks();

    let win = window.as_ref();
    let mut joy_found = 0;
    for i in 0..(JOY_COUNT as u32) {
        let (connected, guid) = is_joy_connected_internal(win, i);
        if connected && register_joystick(joy_state, i, &guid) {
            joy_found += 1;
        }
    }

    linfo!("Found {} valid joysticks.", joy_found);
}

pub fn update_joystick_state(
    joy_state: &mut Joystick_State,
    events: &[Input_Raw_Event],
    window: &Window_Handle,
) {
    update_joysticks();

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

    // TODO: update axes and values
    for (joy_id, joy) in joy_state
        .joysticks
        .iter_mut()
        .filter_map(|j| j.as_ref())
        .enumerate()
    {
        for (axis_id, axis_val) in joy_state.values[joy_id].iter_mut().enumerate() {
            *axis_val = get_joy_axis_value_internal(
                window,
                joy,
                Joystick_Axis::try_from(axis_id as u8).unwrap(),
            );
        }
    }
}

fn register_joystick(
    joy_state: &mut Joystick_State,
    joystick_id: Joystick_Id,
    joystick_guid: &Option<Box<str>>,
) -> bool {
    ldebug!("Registering joystick {}", joystick_id);

    match get_joy_type_internal(joystick_guid) {
        Ok(joy_type) => {
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

#[inline]
fn get_joy_btn_id_xbox360(button: Joystick_Button) -> Option<Button_Id> {
    assert!((button as usize) < BUTTONS_TO_IDS_XBOX360.len());
    BUTTONS_TO_IDS_XBOX360[button as usize]
}

fn get_joy_axis_value_xbox360(
    window: &Window_Handle,
    joystick_id: Joystick_Id,
    axis: Joystick_Axis,
) -> f32 {
    backend::get_joy_axis_value_xbox360(window, joystick_id, axis)
}

#[inline]
fn get_joy_btn_from_id_xbox360(btn_id: Button_Id) -> Option<Joystick_Button> {
    if (btn_id as usize) < IDS_TO_BUTTONS_XBOX360.len() {
        Some(IDS_TO_BUTTONS_XBOX360[btn_id as usize])
    } else {
        None
    }
}

/// Forces to update the joysticks' state
fn update_joysticks() {
    backend::update_joysticks();
}

// Map (Joystick_Button as u8) => (button id)
#[cfg(target_os = "linux")]
const BUTTONS_TO_IDS_XBOX360: [Option<Button_Id>; Joystick_Button::_Count as usize] = [
    Some(3),  // Face_Top
    Some(1),  // Face_Right
    Some(0),  // Face_Bottom
    Some(2),  // Face_Left
    Some(6),  // Special_Left
    Some(7),  // Special_Right
    Some(8),  // Special_Middle
    Some(9),  // Stick_Left
    Some(10), // Stick_Right
    Some(4),  // Shoulder_Left
    Some(5),  // Shoulder_Right
    None,     // Dpad_Top
    None,     // Dpad_Right
    None,     // Dpad_Bottom
    None,     // Dpad_Left
    None,     // Trigger_Left
    None,     // Trigger_Right
];

#[cfg(target_os = "windows")]
const BUTTONS_TO_IDS_XBOX360: [Option<Button_Id>; Joystick_Button::_Count as usize] = [
    Some(3),  // Face_Top
    Some(1),  // Face_Right
    Some(0),  // Face_Bottom
    Some(2),  // Face_Left
    Some(6),  // Special_Left
    Some(7),  // Special_Right
    None,     // Special_Middle
    Some(9),  // Stick_Left
    Some(10), // Stick_Right
    Some(4),  // Shoulder_Left
    Some(5),  // Shoulder_Right
    None,     // Dpad_Top
    None,     // Dpad_Right
    None,     // Dpad_Bottom
    None,     // Dpad_Left
    None,     // Trigger_Left
    None,     // Trigger_Right
];

#[cfg(target_os = "macos")]
const BUTTONS_TO_IDS_XBOX360: [Option<Button_Id>; Joystick_Button::_Count as usize] = [
    Some(3),  // Face_Top
    Some(2),  // Face_Right
    Some(1),  // Face_Bottom
    Some(0),  // Face_Left
    Some(8),  // Special_Left
    Some(9),  // Special_Right
    Some(12), // Special_Middle
    Some(10), // Stick_Left
    Some(11), // Stick_Right
    Some(4),  // Shoulder_Left
    Some(5),  // Shoulder_Right
    None,     // Dpad_Top
    None,     // Dpad_Right
    None,     // Dpad_Bottom
    None,     // Dpad_Left
    Some(6),  // Trigger_Left
    Some(7),  // Trigger_Right
];

#[cfg(target_os = "linux")]
const IDS_TO_BUTTONS_XBOX360: [Joystick_Button; 11] = [
    Joystick_Button::Face_Bottom,
    Joystick_Button::Face_Right,
    Joystick_Button::Face_Left,
    Joystick_Button::Face_Top,
    Joystick_Button::Shoulder_Left,
    Joystick_Button::Shoulder_Right,
    Joystick_Button::Special_Left,
    Joystick_Button::Special_Right,
    Joystick_Button::Special_Middle,
    Joystick_Button::Stick_Left,
    Joystick_Button::Stick_Right,
];

#[cfg(target_os = "windows")]
const IDS_TO_BUTTONS_XBOX360: [Joystick_Button; 10] = [
    Joystick_Button::Face_Bottom,
    Joystick_Button::Face_Right,
    Joystick_Button::Face_Left,
    Joystick_Button::Face_Top,
    Joystick_Button::Shoulder_Left,
    Joystick_Button::Shoulder_Right,
    Joystick_Button::Special_Left,
    Joystick_Button::Special_Right,
    Joystick_Button::Stick_Left,
    Joystick_Button::Stick_Right,
];

#[cfg(target_os = "macos")]
const IDS_TO_BUTTONS_XBOX360: [Joystick_Button; 13] = [
    Joystick_Button::Face_Left,
    Joystick_Button::Face_Bottom,
    Joystick_Button::Face_Right,
    Joystick_Button::Face_Top,
    Joystick_Button::Shoulder_Left,
    Joystick_Button::Shoulder_Right,
    Joystick_Button::Trigger_Left,
    Joystick_Button::Trigger_Right,
    Joystick_Button::Special_Left,
    Joystick_Button::Special_Right,
    Joystick_Button::Stick_Left,
    Joystick_Button::Stick_Right,
    Joystick_Button::Special_Middle,
];

use super::{Joystick_Axis, Joystick_Button, Joystick_Id};
use glfw::{GamepadAxis, GamepadButton};
use inle_core::env::{asset_path, Env_Info};
use inle_win::window::Window_Handle;

pub(super) const JOY_COUNT: u32 = glfw::ffi::JOYSTICK_LAST as u32 + 1;

pub(super) fn get_joy_axis_value_xbox360(
    window: &Window_Handle,
    joystick_id: Joystick_Id,
    axis: Joystick_Axis,
) -> f32 {
    assert!((axis as usize) < AXES_ENGINE_TO_FRAMEWORK_XBOX360.len());
    if let Some(ax) = AXES_ENGINE_TO_FRAMEWORK_XBOX360[axis as usize] {
        let joy = glfw::Joystick {
            glfw: window.glfw.clone(),
            id: engine_to_framework_joy_id(joystick_id),
        };
        if let Some(gamepad_state) = joy.get_gamepad_state() {
            let axis_pos = gamepad_state.get_axis(ax);
            let (min, max) = AXES_RANGES_XBOX360[axis as usize];
            return norm_minus_one_to_one(axis_pos, min, max);
        }
    }

    0.0
}

pub(super) fn is_joy_btn_pressed_internal_xbox360(
    window: &Window_Handle,
    joystick_id: Joystick_Id,
    button: Joystick_Button,
) -> bool {
    assert!((button as usize) < BUTTONS_ENGINE_TO_FRAMEWORK_XBOX360.len());
    if let Some(btn) = BUTTONS_ENGINE_TO_FRAMEWORK_XBOX360[button as usize] {
        let joy = glfw::Joystick {
            glfw: window.glfw.clone(),
            id: engine_to_framework_joy_id(joystick_id),
        };

        if let Some(gamepad_state) = joy.get_gamepad_state() {
            return gamepad_state.get_button_state(btn) == glfw::Action::Press;
        } else {
            lerr!("failed to get gamepad state for joy {:?}", joy);
        }
    }
    false
}

#[inline]
pub(super) fn is_joy_connected_internal(
    window: &Window_Handle,
    id: Joystick_Id,
) -> (bool, Option<Box<str>>) {
    let joy = glfw::Joystick {
        id: engine_to_framework_joy_id(id),
        glfw: window.glfw.clone(),
    };

    (joy.is_gamepad(), joy.get_guid().map(|s| s.into_boxed_str()))
}

fn engine_to_framework_joy_id(id: Joystick_Id) -> glfw::JoystickId {
    debug_assert!(id < glfw::ffi::JOYSTICK_LAST as u32);
    // Note: this is safe because glfw::JoystickId is repr(i32)
    unsafe { std::mem::transmute(id) }
}

#[inline(always)]
fn norm_minus_one_to_one(x: f32, min: f32, max: f32) -> f32 {
    2.0 * (x - min) / (max - min) - 1.0
}

#[inline(always)]
pub(super) fn update_joysticks() {}

pub(super) fn init_joysticks(window: &Window_Handle, env: &Env_Info) {
    let controller_db = asset_path(env, "", "gamecontrollerdb.txt");
    let now = std::time::Instant::now();
    match std::fs::read_to_string(&controller_db) {
        Ok(db) => {
            if window.glfw.update_gamepad_mappings(&db) {
                let took = now.elapsed();
                lok!(
                    "Successfully updated gamepad mappings from {} in {:?}",
                    controller_db.display(),
                    took
                );
            } else {
                lerr!(
                    "Failed to update gamepad mappings from {}",
                    controller_db.display()
                );
            }
        }
        Err(err) => lwarn!(
            "Failed to read controller db from {}: {}",
            controller_db.display(),
            err
        ),
    }
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
const AXES_ENGINE_TO_FRAMEWORK_XBOX360: [Option<GamepadAxis>; Joystick_Axis::_Count as usize] = [
    Some(GamepadAxis::AxisLeftX),        // Stick_Left_H
    Some(GamepadAxis::AxisLeftY),        // Stick_Left_V,
    Some(GamepadAxis::AxisRightX),       // Stick_Right_H,
    Some(GamepadAxis::AxisRightY),       // Stick_Right_V,
    Some(GamepadAxis::AxisLeftTrigger),  // Trigger_Left,
    Some(GamepadAxis::AxisRightTrigger), // Trigger_Right,
    None,
    None,
];

#[cfg(target_os = "linux")]
const AXES_RANGES_XBOX360: [(f32, f32); Joystick_Axis::_Count as usize] = [
    (-1.0, 1.0),
    (-1.0, 1.0),
    (-1.0, 1.0),
    (-1.0, 1.0),
    (-1.0, 1.0),
    (-1.0, 1.0),
    (-1.0, 1.0),
    (-1.0, 1.0),
];

#[cfg(any(target_os = "windows", target_os = "macos"))]
const AXES_RANGES_XBOX360: [(f32, f32); Joystick_Axis::_Count as usize] = [
    (-1.0, 1.0),
    (-1.0, 1.0),
    (-1.0, 1.0),
    (-1.0, 1.0),
    (-1.0, 1.0),
    (-1.0, 1.0),
    (-1.0, 1.0),
    (1.0, -1.0), // The Dpad_V is inverted on Windows and OSX
];

const BUTTONS_ENGINE_TO_FRAMEWORK_XBOX360: [Option<GamepadButton>;
    Joystick_Button::_Count as usize] = [
    Some(GamepadButton::ButtonY),           // Face_Top
    Some(GamepadButton::ButtonB),           // Face_Right
    Some(GamepadButton::ButtonA),           // Face_Bottom
    Some(GamepadButton::ButtonX),           // Face_Left
    Some(GamepadButton::ButtonBack),        // Special_Left
    Some(GamepadButton::ButtonStart),       // Special_Right
    Some(GamepadButton::ButtonGuide),       // Special_Middle
    Some(GamepadButton::ButtonLeftThumb),   // Stick_Left
    Some(GamepadButton::ButtonRightThumb),  // Stick_Right
    Some(GamepadButton::ButtonLeftBumper),  // Shoulder_Left
    Some(GamepadButton::ButtonRightBumper), // Shoulder_Right
    Some(GamepadButton::ButtonDpadUp),      // Dpad_Top
    Some(GamepadButton::ButtonDpadRight),   // Dpad_Right
    Some(GamepadButton::ButtonDpadDown),    // Dpad_Bottom
    Some(GamepadButton::ButtonDpadLeft),    // Dpad_Left
    None,                                   // Trigger_Left
    None,                                   // Trigger_Right
];

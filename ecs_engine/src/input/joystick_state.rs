use super::bindings::joystick::{self, Joystick, Joystick_Axis, Joystick_Id, Joystick_Mask};

const JOY_COUNT: usize = joystick::JOY_COUNT as usize;

pub type Real_Axes_Values = [f32; Joystick_Axis::_Count as usize];

#[derive(Clone, Default, Debug)]
pub struct Joystick_State {
    pub joysticks: [Option<Joystick>; JOY_COUNT],
    pub values: [Real_Axes_Values; JOY_COUNT],
}

pub fn register_joystick(joy_state: &mut Joystick_State, joystick_id: Joystick_Id) -> bool {
    ldebug!("Registering joystick {}", joystick_id);

    match joystick::get_joy_type(joystick_id) {
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

pub fn unregister_joystick(joy_state: &mut Joystick_State, joystick_id: Joystick_Id) {
    ldebug!("Unregistering joystick {}", joystick_id);
    joy_state.joysticks[joystick_id as usize] = None;
}

pub fn init_joysticks(joy_state: &mut Joystick_State) {
    joystick::update_joysticks();

    let mut joy_found = 0;
    for i in 0..(JOY_COUNT as u32) {
        if joystick::is_joy_connected(i) && register_joystick(joy_state, i) {
            joy_found += 1;
        }
    }

    linfo!("Found {} valid joysticks.", joy_found);
}

pub fn joy_get_values(joy_state: &Joystick_State, joy: Joystick) -> Option<&Real_Axes_Values> {
    if joy_state.joysticks[joy.id as usize].is_some() {
        Some(&joy_state.values[joy.id as usize])
    } else {
        None
    }
}

pub fn all_joysticks_values(
    joy_state: &Joystick_State,
) -> (&[Real_Axes_Values; JOY_COUNT], Joystick_Mask) {
    (&joy_state.values, joystick::get_connected_joysticks_mask())
}

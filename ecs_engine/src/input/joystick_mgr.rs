use super::bindings::joystick::{self, Joystick, Joystick_Axis};
use super::provider::Input_Provider;
use crate::core::common::Maybe_Error;
use std::default::Default;

const JOY_COUNT: usize = joystick::JOY_COUNT as usize;

pub type Real_Axes_Values = [f32; Joystick_Axis::_Count as usize];

#[derive(Default)]
pub struct Joystick_Manager {
    joysticks: [Option<Joystick>; JOY_COUNT],
    values: [Real_Axes_Values; JOY_COUNT],
}

impl Joystick_Manager {
    pub fn new() -> Joystick_Manager {
        Joystick_Manager {
            joysticks: Default::default(),
            values: Default::default(),
        }
    }

    pub fn register(&mut self, joystick_id: u32) -> bool {
        match joystick::get_joy_type(joystick_id) {
            Ok(joy_type) => {
                self.joysticks[joystick_id as usize] = Some(Joystick {
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

    pub fn unregister(&mut self, joystick_id: u32) {
        self.joysticks[joystick_id as usize] = None;
    }

    pub fn init(&mut self) -> Maybe_Error {
        joystick::update_joysticks();

        let mut joy_found = 0;
        for i in 0..(JOY_COUNT as u32) {
            if joystick::is_joy_connected(i) && self.register(i) {
                joy_found += 1;
            }
        }

        linfo!("Found {} valid joysticks.", joy_found);

        Ok(())
    }

    pub fn values(&self, joy: Joystick) -> Option<&Real_Axes_Values> {
        if self.joysticks[joy.id as usize].is_some() {
            Some(&self.values[joy.id as usize])
        } else {
            None
        }
    }

    pub fn all_values(&self) -> (&[Real_Axes_Values; JOY_COUNT], u8) {
        (&self.values, joystick::get_connected_joysticks_mask())
    }

    pub(super) fn update_from_input_provider(&mut self, provider: &dyn Input_Provider) {
        provider.get_axes(&mut self.values);
    }

    pub fn get_joystick(&self, id: u32) -> Option<&Joystick> {
        self.joysticks[id as usize].as_ref()
    }
}

use super::replay_data::{Replay_Data, Replay_Data_Iter};
use crate::input::bindings::joystick;
use crate::input::input_system::Input_Raw_Event;
use crate::input::joystick_mgr::Real_Axes_Values;
use crate::input::provider::{Input_Provider, Input_Provider_Input};
use std::convert::TryInto;
use std::vec::Vec;

pub struct Replay_Input_Provider {
    cur_frame: u64,
    replay_data_iter: Replay_Data_Iter,
    depleted: bool,
    events: Vec<Input_Raw_Event>,
    axes: Real_Axes_Values,
}

impl Replay_Input_Provider {
    pub fn new(replay_data: Replay_Data) -> Replay_Input_Provider {
        Replay_Input_Provider {
            cur_frame: 0,
            replay_data_iter: replay_data.into_iter(),
            depleted: false,
            events: vec![],
            axes: Real_Axes_Values::default(),
        }
    }
}

impl Input_Provider for Replay_Input_Provider {
    fn update(&mut self, window: &mut Input_Provider_Input) {
        self.events.clear();
        if self.depleted {
            // Once replay data is depleted, feed regular window events.
            self.default_update(window);
        } else {
            loop {
                if let Some(datum) = self.replay_data_iter.cur() {
                    if self.cur_frame >= datum.frame_number {
                        self.events.extend_from_slice(&datum.events);
                        for i in 0..self.axes.len() {
                            if (datum.axes_mask & (1 << i)) != 0 {
                                self.axes[i] = datum.axes[i];
                            }
                        }
                        self.replay_data_iter.next();
                    } else {
                        break;
                    }
                } else {
                    self.depleted = true;
                    break;
                }
            }

            self.update_core_events(window);

            self.cur_frame += 1;
        }
    }

    fn get_events(&self) -> &[Input_Raw_Event] {
        &self.events
    }

    fn get_axes(&mut self, joystick: joystick::Joystick, axes: &mut Real_Axes_Values) {
        *axes = self.axes;
    }

    fn is_realtime_player_input(&self) -> bool {
        self.depleted
    }
}

impl Replay_Input_Provider {
    fn default_update(&mut self, window: &mut Input_Provider_Input) {
        // @Cutnpaste from Default_Input_Provider
        while let Some(evt) = window.poll_event() {
            self.events.push(evt);
        }

        // @Incomplete :multiple_joysticks:
        let joystick = joystick::Joystick {
            id: 0,
            joy_type: joystick::Joystick_Type::XBox360,
        };
        for i in 0u8..joystick::Joystick_Axis::_Count as u8 {
            let axis = i
                .try_into()
                .unwrap_or_else(|_| panic!("Failed to convert {} to a valid Joystick_Axis!", i));
            self.axes[i as usize] = joystick::get_joy_axis_value(joystick, axis);
        }
    }

    #[cfg(feature = "use-sfml")]
    fn update_core_events(&mut self, window: &mut Input_Provider_Input) {
        use sfml::window::Event;

        while let Some(evt) = window.poll_event() {
            match evt {
                Event::Closed | Event::Resized { .. } => self.events.push(evt),
                _ => (),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::replay_data::Replay_Data_Point;
    use super::*;
    use crate::core::common::direction::Direction_Flags;

    use crate::input::bindings::keymap::sfml::keypressed;
    use crate::input::bindings::mouse::sfml::mousepressed;
    use crate::input::joystick_mgr::Real_Axes_Values;
    use sfml::window::mouse::Button;
    use sfml::window::Event;
    use sfml::window::Key;

    #[test]
    fn poll_replayed_events() {
        let mut window = crate::gfx::window::create_render_window(&(), (1, 1), "test window");
        let evt1 = vec![keypressed(Key::Num0)];
        let evt2 = vec![keypressed(Key::A)];
        let evt3 = vec![keypressed(Key::Z), mousepressed(Button::Left)];
        // @Incomplete: axes
        let axes = Real_Axes_Values::default();
        let replay_data = Replay_Data::new_from_data(
            16,
            &vec![
                Replay_Data_Point::new(0, &evt1, &axes, 0x0),
                Replay_Data_Point::new(0, &evt2, &axes, 0x0),
                Replay_Data_Point::new(3, &evt3, &axes, 0x0),
            ],
        );

        assert_eq!(replay_data.data.len(), 3);

        let mut replay_provider = Replay_Input_Provider::new(replay_data);

        fn all_but_resized(provider: &dyn Input_Provider) -> Vec<&Event> {
            provider
                .get_events()
                .iter()
                .filter(|evt| {
                    if let Event::Resized { .. } = evt {
                        false
                    } else {
                        true
                    }
                })
                .collect()
        }

        // frame 0
        replay_provider.update(&mut window);
        let events = all_but_resized(&replay_provider);
        assert_eq!(events.len(), 2);
        assert_eq!(*events[0], keypressed(Key::Num0));
        assert_eq!(*events[1], keypressed(Key::A));

        // frame 1
        replay_provider.update(&mut window);
        let events = all_but_resized(&replay_provider);
        assert_eq!(events.len(), 0);

        // frame 2
        replay_provider.update(&mut window);
        let events = all_but_resized(&replay_provider);
        assert_eq!(events.len(), 0);

        // frame 3
        replay_provider.update(&mut window);
        let events = all_but_resized(&replay_provider);
        assert_eq!(events.len(), 2);
        assert_eq!(*events[0], keypressed(Key::Z));
        assert_eq!(*events[1], mousepressed(Button::Left));
    }
}

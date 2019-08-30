use super::replay_data::{Replay_Data, Replay_Data_Iter};
use crate::cfg::Cfg_Var;
use crate::input::bindings::joystick;
use crate::input::default_input_provider::Default_Input_Provider;
use crate::input::input_system::Input_Raw_Event;
use crate::input::joystick_mgr::{Joystick_Manager, Real_Axes_Values};
use crate::input::provider::{Input_Provider, Input_Provider_Input};

pub struct Replay_Input_Provider_Config {
    pub disable_input_during_replay: Cfg_Var<bool>,
}

pub struct Replay_Input_Provider {
    cur_frame: u64,
    replay_data_iter: Replay_Data_Iter,
    depleted: bool,
    dip: Default_Input_Provider,
    config: Replay_Input_Provider_Config,
}

impl Replay_Input_Provider {
    pub fn new(
        config: Replay_Input_Provider_Config,
        replay_data: Replay_Data,
    ) -> Replay_Input_Provider {
        Replay_Input_Provider {
            cur_frame: 0,
            replay_data_iter: replay_data.into_iter(),
            depleted: false,
            dip: Default_Input_Provider::default(),
            config,
        }
    }
}

impl Input_Provider for Replay_Input_Provider {
    fn update(&mut self, window: &mut Input_Provider_Input, joy_mgr: &Joystick_Manager) {
        self.dip.events.clear();

        if self.depleted {
            // Once replay data is depleted, feed regular window events.
            self.dip.update(window, joy_mgr);
        } else {
            if *self.config.disable_input_during_replay {
                self.update_core_events(window);
            } else {
                while let Some(evt) = window.poll_event() {
                    self.dip.events.push(evt);
                }
            }

            loop {
                if let Some(datum) = self.replay_data_iter.cur() {
                    if self.cur_frame >= datum.frame_number {
                        // We have a new replay data point at this frame.

                        // Update events
                        self.dip.events.extend_from_slice(&datum.events);

                        // Update all joysticks values
                        for joy_id in 0..self.dip.axes.len() {
                            if (datum.joy_mask & (1 << joy_id)) != 0 {
                                let joy_axes = &mut self.dip.axes[joy_id];
                                let joy_data = datum.joy_data[joy_id];
                                for (axis_id, axis) in joy_axes.iter_mut().enumerate() {
                                    if (joy_data.axes_mask & (1 << axis_id)) != 0 {
                                        *axis = joy_data.axes[axis_id];
                                    }
                                }
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

            self.cur_frame += 1;
        }
    }

    fn get_events(&self) -> &[Input_Raw_Event] {
        self.dip.get_events()
    }

    fn get_axes(&self, axes: &mut [Real_Axes_Values; joystick::JOY_COUNT as usize]) {
        self.dip.get_axes(axes)
    }

    fn is_realtime_player_input(&self) -> bool {
        self.depleted
    }
}

impl Replay_Input_Provider {
    #[cfg(feature = "use-sfml")]
    fn update_core_events(&mut self, window: &mut Input_Provider_Input) {
        use sfml::window::Event;

        while let Some(evt) = window.poll_event() {
            match evt {
                Event::Closed | Event::Resized { .. } => self.dip.events.push(evt),
                _ => (),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::replay_data::{Replay_Data_Point, Replay_Joystick_Data};
    use super::*;

    use crate::input::bindings::keymap::sfml::keypressed;
    use crate::input::bindings::mouse::sfml::mousepressed;
    use crate::input::joystick_mgr::Joystick_Manager;
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
        let joy_data: [Replay_Joystick_Data; joystick::JOY_COUNT as usize] =
            std::default::Default::default();
        let replay_data = Replay_Data::new_from_data(
            16,
            &vec![
                Replay_Data_Point::new(0, &evt1, &joy_data, 0x0),
                Replay_Data_Point::new(0, &evt2, &joy_data, 0x0),
                Replay_Data_Point::new(3, &evt3, &joy_data, 0x0),
            ],
        );

        assert_eq!(replay_data.data.len(), 3);

        let mut replay_provider = Replay_Input_Provider::new(
            Replay_Input_Provider_Config {
                disable_input_during_replay: Cfg_Var::new_from_val(true),
            },
            replay_data,
        );

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

        let joy_mgr = Joystick_Manager::new();

        // frame 0
        replay_provider.update(&mut window, &joy_mgr);
        let events = all_but_resized(&replay_provider);
        assert_eq!(events.len(), 2);
        assert_eq!(*events[0], keypressed(Key::Num0));
        assert_eq!(*events[1], keypressed(Key::A));

        // frame 1
        replay_provider.update(&mut window, &joy_mgr);
        let events = all_but_resized(&replay_provider);
        assert_eq!(events.len(), 0);

        // frame 2
        replay_provider.update(&mut window, &joy_mgr);
        let events = all_but_resized(&replay_provider);
        assert_eq!(events.len(), 0);

        // frame 3
        replay_provider.update(&mut window, &joy_mgr);
        let events = all_but_resized(&replay_provider);
        assert_eq!(events.len(), 2);
        assert_eq!(*events[0], keypressed(Key::Z));
        assert_eq!(*events[1], mousepressed(Button::Left));
    }
}

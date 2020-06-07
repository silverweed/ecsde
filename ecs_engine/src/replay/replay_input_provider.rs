use super::replay_data::{Replay_Data, Replay_Data_Iter};
use crate::input::input_state::Input_Raw_State;
use std::iter::Peekable;

pub struct Replay_Input_Provider {
    replay_data_iter: Peekable<Replay_Data_Iter>,
}

impl Replay_Input_Provider {
    pub fn new(replay_data: Replay_Data) -> Self {
        Self {
            replay_data_iter: replay_data.into_iter().peekable(),
        }
    }

    pub fn has_more_input(&mut self) -> bool {
        self.replay_data_iter.peek().is_some()
    }

    pub fn get_replayed_input_for_frame(&mut self, cur_frame: u64) -> Option<Input_Raw_State> {
        if let Some(datum) = self.replay_data_iter.peek() {
            if cur_frame == datum.frame_number {
                // We have a new replay data point at this frame.
                let replay_data = self.replay_data_iter.next().unwrap();
                Some(replay_data.into())
            } else {
                assert!(cur_frame < datum.frame_number);
                None
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::replay_data::{Replay_Data_Point, Replay_Joystick_Data};
    use super::*;
    use crate::test_common::create_test_resources_and_env;

    use crate::input::bindings::keyboard::sfml::keypressed;
    use crate::input::bindings::mouse::sfml::mousepressed;
    use crate::input::joystick_state::Joystick_State;
    use sfml::window::mouse::Button;
    use sfml::window::Event;
    use sfml::window::Key;

    #[test]
    #[ignore] // because create_render_window is expensive and may crash in multithreaded tests
    fn poll_replayed_events() {
        use crate::gfx::window;
        let wc_args = window::Create_Window_Args::default();
        let mut window = crate::gfx::window::create_window(&wc_args, (1, 1), "test window");
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
        let (_, _, env) = create_test_resources_and_env();
        let mut config = cfg::Config::new_from_dir(env.get_test_cfg_root());

        assert_eq!(replay_data.data.len(), 3);

        let mut replay_provider = Replay_Input_Provider::new(
            Replay_Input_Provider_Config {
                disable_input_during_replay: Cfg_Var::new_from_val(true, &mut config),
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

        let joy_mgr = Joystick_State::default();

        // frame 0
        replay_provider.update(&mut window, Some(&joy_mgr), &config);
        let events = all_but_resized(&replay_provider);
        assert_eq!(events.len(), 2);
        assert_eq!(*events[0], keypressed(Key::Num0));
        assert_eq!(*events[1], keypressed(Key::A));

        // frame 1
        replay_provider.update(&mut window, Some(&joy_mgr), &config);
        let events = all_but_resized(&replay_provider);
        assert_eq!(events.len(), 0);

        // frame 2
        replay_provider.update(&mut window, Some(&joy_mgr), &config);
        let events = all_but_resized(&replay_provider);
        assert_eq!(events.len(), 0);

        // frame 3
        replay_provider.update(&mut window, Some(&joy_mgr), &config);
        let events = all_but_resized(&replay_provider);
        assert_eq!(events.len(), 2);
        assert_eq!(*events[0], keypressed(Key::Z));
        assert_eq!(*events[1], mousepressed(Button::Left));
    }
}

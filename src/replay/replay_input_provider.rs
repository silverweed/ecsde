use super::replay_data::{Replay_Data, Replay_Data_Iter};
use crate::input::provider::{Input_Provider, Input_Provider_Input, Input_Provider_Output};
use std::vec::Vec;

pub struct Replay_Input_Provider {
    cur_frame: u64,
    replay_data_iter: Replay_Data_Iter,
    depleted: bool,
}

impl Replay_Input_Provider {
    pub fn new(replay_data: Replay_Data) -> Replay_Input_Provider {
        Replay_Input_Provider {
            cur_frame: 0,
            replay_data_iter: replay_data.into_iter(),
            depleted: false,
        }
    }
}

impl Input_Provider for Replay_Input_Provider {
    fn poll_events(&mut self, window: &mut Input_Provider_Input) -> Vec<Input_Provider_Output> {
        let mut events = vec![];

        if self.depleted {
            // Once replay data is depleted, feed regular window events.
            while let Some(evt) = window.poll_event() {
                events.push(evt);
            }
        } else {
            loop {
                if let Some(datum) = self.replay_data_iter.cur() {
                    if self.cur_frame >= datum.frame_number {
                        events.extend_from_slice(&datum.events);
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

        events
    }

    fn is_realtime_player_input(&self) -> bool {
        self.depleted
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
    use sfml::window::Key;

    #[test]
    fn poll_replayed_events() {
        let mut window = crate::gfx::window::create_render_window(&(), (1, 1), "test window");
        // @Incomplete
        let evt1 = vec![keypressed(Key::Num0)];
        let evt2 = vec![keypressed(Key::A)];
        let evt3 = vec![keypressed(Key::Z), mousepressed(Button::Left)];
        let axes = Real_Axes_Values::default();
        let replay_data = Replay_Data::new_from_data(
            16,
            &vec![
                Replay_Data_Point::new(0, &evt1, &axes),
                Replay_Data_Point::new(0, &evt2, &axes),
                Replay_Data_Point::new(3, &evt3, &axes),
            ],
        );

        assert_eq!(replay_data.data.len(), 3);

        let mut replay_provider = Replay_Input_Provider::new(replay_data);

        // frame 0
        let events = replay_provider.poll_events(&mut window);
        assert_eq!(events.len(), 2);
        assert_eq!(events[0], keypressed(Key::Num0));
        assert_eq!(events[1], keypressed(Key::A));

        // frame 1
        let events = replay_provider.poll_events(&mut window);
        assert_eq!(events.len(), 0);

        // frame 2
        let events = replay_provider.poll_events(&mut window);
        assert_eq!(events.len(), 0);

        // frame 3
        let events = replay_provider.poll_events(&mut window);
        assert_eq!(events.len(), 2);
        assert_eq!(events[0], keypressed(Key::Z));
        assert_eq!(events[1], mousepressed(Button::Left));
    }
}

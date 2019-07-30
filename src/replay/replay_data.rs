use crate::core::common::direction::Direction_Flags;
use crate::core::time;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::time::Duration;
use std::vec::Vec;

#[cfg(feature = "use-sfml")]
type Event_Type = sfml::window::Event;

/// Contains the replay data for a single frame. It consists in time information (a frame number)
/// plus the diff from the previous saved point.
/// @Incomplete: currently only contains data for the four directions (in boolean format).
#[derive(Clone, Debug, PartialEq)]
pub struct Replay_Data_Point {
    frame_number: u64,
    directions: Direction_Flags,
    // @Incomplete :replay_actions:
    actions: Vec<Event_Type>,
}

impl std::default::Default for Replay_Data_Point {
    fn default() -> Replay_Data_Point {
        Replay_Data_Point {
            frame_number: 0,
            directions: Direction_Flags::empty(),
            actions: vec![],
        }
    }
}

impl Replay_Data_Point {
    pub fn new(
        frame_number: u64,
        directions_pressed: Direction_Flags,
        actions: &[Event_Type],
    ) -> Replay_Data_Point {
        Replay_Data_Point {
            frame_number,
            directions: directions_pressed,
            actions: actions.to_vec(),
        }
    }

    pub fn directions(&self) -> Direction_Flags {
        self.directions
    }

    pub fn frame_number(&self) -> u64 {
        self.frame_number
    }

    pub fn actions(&self) -> &[Event_Type] {
        &self.actions
    }
}

#[derive(Debug)]
pub struct Replay_Data {
    pub(self) data: Vec<Replay_Data_Point>,
    /// Note: for the replay to work correctly, the game tick time should not be changed while recording
    pub(self) ms_per_frame: u16,
    pub(self) duration: Duration,
}

impl Replay_Data {
    pub fn new(ms_per_frame: u16) -> Replay_Data {
        Replay_Data {
            data: vec![],
            ms_per_frame,
            duration: Duration::new(0, 0),
        }
    }

    #[cfg(test)]
    pub fn new_from_data(ms_per_frame: u16, data: &[Replay_Data_Point]) -> Replay_Data {
        let mut replay = Replay_Data {
            data: data.to_vec(),
            ms_per_frame,
            duration: Duration::new(0, 0),
        };
        replay.duration = Self::calc_duration(&replay);
        replay
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn duration(&self) -> Duration {
        self.duration
    }

    pub fn from_serialized(path: &Path) -> Result<Replay_Data, Box<dyn std::error::Error>> {
        let now = std::time::SystemTime::now();
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let replay = Self::deserialize(&content);
        let time_elapsed = std::time::SystemTime::now().duration_since(now).unwrap();
        eprintln!(
            "[ OK ] Loaded replay data from {:?} in {} ms. Replay duration = {} s.",
            path,
            time_elapsed.as_millis(),
            time::to_secs_frac(&replay.duration)
        );
        Ok(replay)
    }

    pub fn deserialize(content: &str) -> Replay_Data {
        let mut replay = Replay_Data::new(0);

        // First line should contains the ms per frame
        let mut lines = content.lines();
        if let Some(line) = lines.next() {
            replay.ms_per_frame = match line.trim().parse::<u16>() {
                Ok(ms_per_frame) => ms_per_frame,
                Err(_err) => {
                    eprintln!("[ WARNING ] Error parsing ms_per_frame: line was {}", line);
                    0
                }
            }
        }

        for line in lines {
            let tokens: Vec<_> = line.splitn(2, ' ').collect();
            if tokens.len() == 2 {
                match tokens[0].parse::<u64>() {
                    Ok(frame_number) => match tokens[1].parse::<u8>() {
                        Ok(directions) => replay.data.push(Replay_Data_Point::new(
                            frame_number,
                            Direction_Flags::from_bits_truncate(directions),
                            &vec![], // @Incomplete :replay_actions:
                        )),
                        Err(_err) => {
                            eprintln!(
                                "[ WARNING ] Error parsing directions: token was {}",
                                tokens[1]
                            );
                            continue;
                        }
                    },
                    Err(_err) => {
                        eprintln!(
                            "[ WARNING ] Error parsing frame number: token was {}",
                            tokens[0]
                        );
                        continue;
                    }
                }
            } else {
                eprintln!("[ WARNING ] Bogus line in replay data: {}", line);
            }
        }

        replay.duration = Self::calc_duration(&replay);

        replay
    }

    #[inline]
    fn calc_duration(replay: &Replay_Data) -> Duration {
        let last_frame_number = replay.data[replay.data.len() - 1].frame_number();
        Duration::from_millis(last_frame_number * u64::from(replay.ms_per_frame))
    }

    pub fn add_point(
        &mut self,
        frame_number: u64,
        directions: Direction_Flags,
        actions: &[Event_Type],
    ) {
        self.data
            .push(Replay_Data_Point::new(frame_number, directions, actions));
    }

    pub fn serialize(&self) -> String {
        let mut s = String::from("");

        s.push_str(&self.ms_per_frame.to_string());
        s.push_str("\r\n");

        // @Incomplete: for now, serialize plain text. Later, do binary.
        for datum in self.data.iter() {
            s.push_str(datum.frame_number.to_string().as_str());
            s.push(' ');
            s.push_str(datum.directions.bits().to_string().as_str());
            s.push_str("\r\n");
        }

        s
    }
}

impl std::iter::IntoIterator for Replay_Data {
    type Item = Replay_Data_Point;
    type IntoIter = Replay_Data_Iter;

    fn into_iter(self) -> Self::IntoIter {
        Replay_Data_Iter {
            replay: self,
            idx: 0,
        }
    }
}

pub struct Replay_Data_Iter {
    pub(self) replay: Replay_Data,
    pub(self) idx: usize,
}

impl std::iter::Iterator for Replay_Data_Iter {
    type Item = Replay_Data_Point;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx == self.replay.data.len() {
            None
        } else {
            let mut item = Replay_Data_Point::default();
            std::mem::swap(&mut item, &mut self.replay.data[self.idx]);
            self.idx += 1;
            Some(item)
        }
    }
}

impl Replay_Data_Iter {
    pub fn cur(&self) -> Option<&Replay_Data_Point> {
        if self.idx == self.replay.data.len() {
            None
        } else {
            Some(&self.replay.data[self.idx])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_deserialize() {
        // @Incomplete :replay_actions:
        let data_points = vec![
            Replay_Data_Point::new(0, Direction_Flags::UP, &vec![]),
            Replay_Data_Point::new(1, Direction_Flags::RIGHT, &vec![]),
            Replay_Data_Point::new(10, Direction_Flags::UP | Direction_Flags::DOWN, &vec![]),
            Replay_Data_Point::new(209, Direction_Flags::LEFT, &vec![]),
            Replay_Data_Point::new(
                1110,
                Direction_Flags::LEFT | Direction_Flags::RIGHT,
                &vec![],
            ),
            Replay_Data_Point::new(
                1111,
                Direction_Flags::UP
                    | Direction_Flags::RIGHT
                    | Direction_Flags::DOWN
                    | Direction_Flags::LEFT,
                &vec![],
            ),
            Replay_Data_Point::new(
                6531,
                Direction_Flags::UP | Direction_Flags::LEFT | Direction_Flags::DOWN,
                &vec![],
            ),
            Replay_Data_Point::new(
                424242,
                Direction_Flags::DOWN | Direction_Flags::RIGHT,
                &vec![],
            ),
        ];

        let replay = Replay_Data::new_from_data(10, &data_points);
        let serialized = replay.serialize();
        let deserialized = Replay_Data::deserialize(&serialized);

        assert_eq!(deserialized.ms_per_frame, replay.ms_per_frame);
        assert_eq!(deserialized.duration, replay.duration);
        assert_eq!(deserialized.data.len(), replay.data.len());
        for i in 0..replay.data.len() {
            assert_eq!(deserialized.data[i], replay.data[i]);
        }
    }
}

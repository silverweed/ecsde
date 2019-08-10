use crate::core::common::direction::Direction_Flags;
use crate::core::common::serialize::Serializable;
use crate::core::common::stringid::String_Id;
use crate::core::time;
use crate::input::axes::Virtual_Axes;
use crate::input::bindings::joystick;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::time::Duration;
use std::vec::Vec;

#[cfg(feature = "use-sfml")]
type Event_Type = sfml::window::Event;

const AXES_COUNT: usize = joystick::Joystick_Axis::_Count as usize;

/// Contains the replay data for a single frame. It consists in time information (a frame number)
/// plus the diff from the previous saved point.
/// Note that raw events and real axes, rather than processed game actions or virtual axes, are saved.
#[derive(Clone, Debug, PartialEq)]
pub struct Replay_Data_Point {
    pub frame_number: u64,
    pub actions: Vec<Event_Type>,
    pub axes: [f32; AXES_COUNT],
}

impl std::default::Default for Replay_Data_Point {
    fn default() -> Replay_Data_Point {
        let axes: [f32; AXES_COUNT] = std::default::Default::default();
        Replay_Data_Point {
            frame_number: 0,
            actions: vec![],
            axes,
        }
    }
}

impl Replay_Data_Point {
    pub fn new(
        frame_number: u64,
        actions: &[Event_Type],
        axes: &[f32; AXES_COUNT],
    ) -> Replay_Data_Point {
        Replay_Data_Point {
            frame_number,
            actions: actions.to_vec(),
            axes: axes.clone(),
        }
    }
}

impl Serializable for Replay_Data_Point {
    fn serialize(&self) -> String {
        // @Temporary
        format!("{} {:?} {:?}", self.frame_number, self.actions, self.axes)
    }

    fn deserialize(raw: &str) -> Result<Replay_Data_Point, String> {
        // @Temporary
        let axes: [f32; AXES_COUNT] = std::default::Default::default();
        Ok(Replay_Data_Point::new(0, &[], &axes))
    }
}

#[derive(Debug)]
pub struct Replay_Data {
    pub data: Vec<Replay_Data_Point>,
    /// Note: for the replay to work correctly, the game tick time should not be changed while recording
    pub ms_per_frame: u16,
    pub duration: Duration,
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

    pub fn from_file(path: &Path) -> Result<Replay_Data, Box<dyn std::error::Error>> {
        let now = std::time::SystemTime::now();
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let replay = Self::deserialize(&content)?;
        let time_elapsed = std::time::SystemTime::now().duration_since(now).unwrap();
        eprintln!(
            "[ OK ] Loaded replay data from {:?} in {} ms. Replay duration = {} s.",
            path,
            time_elapsed.as_millis(),
            time::to_secs_frac(&replay.duration)
        );
        Ok(replay)
    }

    #[inline]
    fn calc_duration(replay: &Replay_Data) -> Duration {
        if replay.data.len() == 0 {
            Duration::new(0, 0)
        } else {
            let last_frame_number = replay.data[replay.data.len() - 1].frame_number;
            Duration::from_millis(last_frame_number * u64::from(replay.ms_per_frame))
        }
    }
}

impl Serializable for Replay_Data {
    fn serialize(&self) -> String {
        let mut s = String::from("");

        s.push_str(&self.ms_per_frame.to_string());
        s.push_str("\r\n");

        // @Incomplete: for now, serialize plain text. Later, do binary.
        for datum in self.data.iter() {
            s.push_str(datum.frame_number.to_string().as_str());
            s.push(' ');
            //s.push_str(datum.directions.bits().to_string().as_str());
            s.push_str("\r\n");
        }

        s
    }

    fn deserialize(raw: &str) -> Result<Replay_Data, String> {
        let mut replay = Replay_Data::new(0);

        // First line should contains the ms per frame
        let mut lines = raw.lines();
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
            match Replay_Data_Point::deserialize(line) {
                Ok(point) => replay.data.push(point),
                Err(msg) => eprintln!("[ WARNING ] Error parsing line {}: {}", line, msg),
            }
        }

        replay.duration = Self::calc_duration(&replay);

        Ok(replay)
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

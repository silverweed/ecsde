use crate::core::common::serialize::{Binary_Serializable, Byte_Stream};
use crate::core::time;
use crate::input::bindings::joystick;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::time::Duration;
use std::vec::Vec;

#[cfg(feature = "use-sfml")]
type Event_Type = ::sfml::window::Event;

const AXES_COUNT: usize = joystick::Joystick_Axis::_Count as usize;

/// Contains the replay data for a single frame. It consists in time information (a frame number)
/// plus the diff from the previous saved point.
/// Note that raw events and real axes, rather than processed game actions or virtual axes, are saved.
#[derive(Clone, Debug, PartialEq)]
pub struct Replay_Data_Point {
    pub frame_number: u64,
    pub events: Vec<Event_Type>,
    pub axes: [f32; AXES_COUNT],
    /// Bitmask indicating which axes in self.axes must be considered.
    /// This is done for optimizing the disk space taken by serialized replay data:
    /// we don't serialize unchanged axes values.
    pub axes_mask: u8,
}

impl std::default::Default for Replay_Data_Point {
    fn default() -> Replay_Data_Point {
        let axes: [f32; AXES_COUNT] = std::default::Default::default();
        Replay_Data_Point {
            frame_number: 0,
            events: vec![],
            axes,
            axes_mask: u8::max_value(),
        }
    }
}

impl Replay_Data_Point {
    pub fn new(
        frame_number: u64,
        events: &[Event_Type],
        axes: &[f32; AXES_COUNT],
        axes_mask: u8,
    ) -> Replay_Data_Point {
        Replay_Data_Point {
            frame_number,
            events: events.to_vec(),
            axes: *axes,
            axes_mask,
        }
    }
}

impl Binary_Serializable for Replay_Data_Point {
    fn serialize(&self, output: &mut Byte_Stream) -> std::io::Result<()> {
        output.write_u32(self.frame_number as u32)?;

        output.write_u8(self.events.len() as u8)?;
        for event in self.events.iter() {
            event.serialize(output)?;
        }

        output.write_u8(self.axes_mask)?;
        for i in 0..AXES_COUNT {
            if (self.axes_mask & (1 << i)) != 0 {
                output.write_u32(self.axes[i].to_bits())?;
            }
        }

        Ok(())
    }

    fn deserialize(input: &mut Byte_Stream) -> std::io::Result<Replay_Data_Point> {
        let frame_number = input.read_u32()? as u64;

        let n_events = input.read_u8()?;
        let mut events = vec![];
        for _ in 0u8..n_events {
            events.push(Event_Type::deserialize(input)?);
        }

        let mut axes: [f32; AXES_COUNT] = std::default::Default::default();
        let axes_mask = input.read_u8()?;
        for (i, axis) in axes.iter_mut().enumerate() {
            if (axes_mask & (1 << i)) != 0 {
                let val = input.read_u32()?;
                let val = f32::from_bits(val);
                *axis = val;
            }
        }

        Ok(Replay_Data_Point::new(
            frame_number,
            &events,
            &axes,
            axes_mask,
        ))
    }
}

/// Replay_Data is used only for the playback. It loads serialized replay data from a file
/// and provides an iterator to access all the recorded events.
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

        let mut buf = vec![];
        file.read_to_end(&mut buf)?;

        let mut byte_stream = Byte_Stream::new_from_vec(buf);
        let replay = Self::deserialize(&mut byte_stream)?;

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
        if replay.data.is_empty() {
            Duration::new(0, 0)
        } else {
            let last_frame_number = replay.data[replay.data.len() - 1].frame_number;
            Duration::from_millis(last_frame_number * u64::from(replay.ms_per_frame))
        }
    }
}

impl Binary_Serializable for Replay_Data {
    fn deserialize(input: &mut Byte_Stream) -> std::io::Result<Replay_Data> {
        let mut replay = Replay_Data::new(0);

        // First line should contains the ms per frame
        replay.ms_per_frame = input.read_u16()?;

        while (input.pos() as usize) < input.len() {
            replay.data.push(Replay_Data_Point::deserialize(input)?);
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
    use crate::input::joystick_mgr::Real_Axes_Values;

    #[test]
    fn serialize_deserialize_replay_data_point() {
        // @Incomplete :replay_actions:
        let axes = Real_Axes_Values::default();
        let data_points = vec![
            Replay_Data_Point::new(0, &[], &axes, 0x0),
            Replay_Data_Point::new(1, &[], &axes, 0x0),
            Replay_Data_Point::new(10, &[], &axes, 0x0),
            Replay_Data_Point::new(209, &[], &axes, 0x0),
            Replay_Data_Point::new(1110, &[], &axes, 0x0),
            Replay_Data_Point::new(1111, &[], &axes, 0x0),
            Replay_Data_Point::new(6531, &[], &axes, 0x0),
            Replay_Data_Point::new(424242, &[], &axes, 0x0),
        ];

        let mut byte_stream = Byte_Stream::new();

        for point in data_points.iter() {
            point
                .serialize(&mut byte_stream)
                .unwrap_or_else(|err| panic!("Error serializing replay data point: {}", err));
        }

        byte_stream.seek(0);

        let mut deser_points = vec![];

        while (byte_stream.pos() as usize) < byte_stream.len() {
            println!("pos: {}/{}", byte_stream.pos(), byte_stream.len());
            deser_points.push(
                Replay_Data_Point::deserialize(&mut byte_stream).unwrap_or_else(|err| {
                    panic!("Failed to deserialize replay data point: {}", err)
                }),
            );
        }

        assert_eq!(data_points.len(), deser_points.len());
        for i in 0..data_points.len() {
            assert_eq!(data_points[i], deser_points[i]);
        }
    }

    #[test]
    fn serialize_deserialize_replay_data() {
        // @Incomplete :replay_actions:
        let axes = Real_Axes_Values::default();
        let data_points = vec![
            Replay_Data_Point::new(0, &[], &axes, 0x0),
            Replay_Data_Point::new(1, &[], &axes, 0x0),
            Replay_Data_Point::new(10, &[], &axes, 0x0),
            Replay_Data_Point::new(209, &[], &axes, 0x0),
            Replay_Data_Point::new(1110, &[], &axes, 0x0),
            Replay_Data_Point::new(1111, &[], &axes, 0x0),
            Replay_Data_Point::new(6531, &[], &axes, 0x0),
            Replay_Data_Point::new(424242, &[], &axes, 0x0),
        ];

        let mut byte_stream = Byte_Stream::new();

        // Simulate the serialization done by the recording thread
        let ms_per_frame = 16;
        byte_stream.write_u16(ms_per_frame).unwrap();

        for point in data_points.iter() {
            point
                .serialize(&mut byte_stream)
                .unwrap_or_else(|err| panic!("Failed to serialize replay data point: {}", err));
        }

        byte_stream.seek(0);

        let deserialized = Replay_Data::deserialize(&mut byte_stream)
            .unwrap_or_else(|err| panic!("Failed to deserialize replay data: {}", err));

        assert_eq!(deserialized.ms_per_frame, ms_per_frame);
        assert_eq!(
            deserialized.duration,
            Duration::from_millis(424242u64 * (ms_per_frame as u64))
        );
        assert_eq!(deserialized.data.len(), data_points.len());
        for i in 0..data_points.len() {
            assert_eq!(deserialized.data[i], data_points[i]);
        }
    }
}

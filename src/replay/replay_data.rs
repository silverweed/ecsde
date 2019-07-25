use crate::core::common::direction::Direction_Flags;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::vec::Vec;

/// Contains the replay data for a single frame. It consists in time information (a frame number)
/// plus the diff from the previous saved point.
/// @Incomplete: currently only contains data for the four directions (in boolean format).
#[derive(Copy, Clone, Debug)]
pub struct Replay_Data_Point {
    frame_number: u64,
    directions: Direction_Flags,
}

impl Replay_Data_Point {
    pub fn new(frame_number: u64, directions_pressed: Direction_Flags) -> Replay_Data_Point {
        Replay_Data_Point {
            frame_number,
            directions: directions_pressed,
        }
    }

    pub fn directions(&self) -> Direction_Flags {
        self.directions
    }

    pub fn frame_number(&self) -> u64 {
        self.frame_number
    }
}

pub struct Replay_Data {
    pub(self) data: Vec<Replay_Data_Point>,
}

impl Replay_Data {
    pub fn new() -> Replay_Data {
        Replay_Data { data: vec![] }
    }

    pub fn from_serialized(path: &Path) -> Result<Replay_Data, Box<dyn std::error::Error>> {
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        let mut replay = Replay_Data::new();

        for line in content.lines() {
            let tokens: Vec<_> = line.splitn(2, ' ').collect();
            if tokens.len() == 2 {
                replay.data.push(Replay_Data_Point::new(
                    tokens[0].parse::<u64>()?,
                    Direction_Flags::from_bits_truncate(tokens[1].parse::<u8>()?),
                ));
            } else {
                eprintln!("[ WARNING ] Bogus line in replay data: {}", line);
            }
        }

        eprintln!("REPLAY_DATA: {:?}", replay.data);

        Ok(replay)
    }

    pub fn add_point(&mut self, frame_number: u64, directions: Direction_Flags) {
        self.data
            .push(Replay_Data_Point::new(frame_number, directions));
    }

    pub fn serialize(&self) -> String {
        let mut s = String::from("");

        // @Incomplete: for now, serialize plain text. Later, do binary.
        for datum in self.data.iter() {
            s.push_str(datum.frame_number.to_string().as_str());
            s.push(' ');
            s.push_str(datum.directions.bits().to_string().as_str());
            s.push_str("\r\n");
        }

        s
    }

    pub fn iter<'a>(&'a self) -> Replay_Data_Iter<'a> {
        Replay_Data_Iter {
            replay: self,
            idx: 0,
        }
    }
}

pub struct Replay_Data_Iter<'a> {
    pub(self) replay: &'a Replay_Data,
    pub(self) idx: usize,
}

impl std::iter::Iterator for Replay_Data_Iter<'_> {
    type Item = Replay_Data_Point;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx == self.replay.data.len() {
            None
        } else {
            let item = self.replay.data[self.idx];
            self.idx += 1;
            Some(item)
        }
    }
}

impl Replay_Data_Iter<'_> {
    pub fn cur(&self) -> Option<&Replay_Data_Point> {
        if self.idx == self.replay.data.len() {
            None
        } else {
            Some(&self.replay.data[self.idx])
        }
    }
}

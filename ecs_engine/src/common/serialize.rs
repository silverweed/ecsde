use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::Cursor;
use std::vec::Vec;

pub trait Binary_Serializable: Sized {
    fn serialize(&self, _output: &mut Byte_Stream) -> std::io::Result<()> {
        unimplemented!();
    }

    fn deserialize(_input: &mut Byte_Stream) -> std::io::Result<Self> {
        unimplemented!();
    }
}

#[derive(Default)]
pub struct Byte_Stream {
    cursor: Cursor<Vec<u8>>,
}

impl std::convert::AsRef<[u8]> for Byte_Stream {
    fn as_ref(&self) -> &[u8] {
        self.cursor.get_ref()
    }
}

impl Byte_Stream {
    pub fn new() -> Byte_Stream {
        Byte_Stream {
            cursor: Cursor::new(vec![]),
        }
    }

    pub fn new_from_vec(data: Vec<u8>) -> Byte_Stream {
        Byte_Stream {
            cursor: Cursor::new(data),
        }
    }

    pub fn seek(&mut self, pos: u64) {
        self.cursor.set_position(pos);
    }

    pub fn pos(&self) -> u64 {
        self.cursor.position()
    }

    pub fn len(&self) -> usize {
        self.cursor.get_ref().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn write_u8(&mut self, x: u8) -> std::io::Result<()> {
        self.cursor.write_u8(x)
    }

    pub fn write_u16(&mut self, x: u16) -> std::io::Result<()> {
        self.cursor.write_u16::<LittleEndian>(x)
    }

    pub fn write_u32(&mut self, x: u32) -> std::io::Result<()> {
        self.cursor.write_u32::<LittleEndian>(x)
    }

    pub fn write_u64(&mut self, x: u64) -> std::io::Result<()> {
        self.cursor.write_u64::<LittleEndian>(x)
    }

    pub fn write_f32(&mut self, x: f32) -> std::io::Result<()> {
        let x_as_le = x.to_le_bytes();
        for &b in &x_as_le {
            self.cursor.write_u8(b)?;
        }
        Ok(())
    }

    pub fn read_u8(&mut self) -> std::io::Result<u8> {
        self.cursor.read_u8()
    }

    pub fn read_u16(&mut self) -> std::io::Result<u16> {
        self.cursor.read_u16::<LittleEndian>()
    }

    pub fn read_u32(&mut self) -> std::io::Result<u32> {
        self.cursor.read_u32::<LittleEndian>()
    }

    pub fn read_u64(&mut self) -> std::io::Result<u64> {
        self.cursor.read_u64::<LittleEndian>()
    }

    pub fn read_f32(&mut self) -> std::io::Result<f32> {
        let mut x_as_le = [0u8; 4];
        for byte in &mut x_as_le {
            *byte = self.cursor.read_u8()?;
        }
        Ok(f32::from_le_bytes(x_as_le))
    }
}

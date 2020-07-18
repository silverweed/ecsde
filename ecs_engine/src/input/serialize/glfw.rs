use crate::common::serialize::{Binary_Serializable, Byte_Stream};
use crate::gfx::window::Event;

pub fn should_event_be_serialized(evt: &Event) -> bool {
    // @Incomplete
    false
}

impl Binary_Serializable for Event {
    fn serialize(&self, _output: &mut Byte_Stream) -> std::io::Result<()> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            String::from("unimplemented!"),
        ))
    }

    fn deserialize(_input: &mut Byte_Stream) -> std::io::Result<Self> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            String::from("unimplemented!"),
        ))
    }
}

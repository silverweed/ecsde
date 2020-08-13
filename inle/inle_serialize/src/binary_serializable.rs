use super::byte_stream::Byte_Stream;

pub trait Binary_Serializable: Sized {
    fn serialize(&self, _output: &mut Byte_Stream) -> std::io::Result<()> {
        unimplemented!();
    }

    fn deserialize(_input: &mut Byte_Stream) -> std::io::Result<Self> {
        unimplemented!();
    }
}

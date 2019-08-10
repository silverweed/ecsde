pub trait Serializable: Sized {
    // @Temporary
    fn serialize(&self) -> String;
    fn deserialize(raw: &str) -> Result<Self, String>;
}

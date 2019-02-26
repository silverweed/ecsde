use super::common;

pub trait System {
    type Config;

    fn init(&mut self, _: &Self::Config) -> common::Maybe_Error {
        Ok(())
    }

    fn update(&mut self, delta: &std::time::Duration);
}

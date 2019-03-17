use super::common;

pub trait System {
    type Config;
    type Update_Params;

    fn init(&mut self, _: Self::Config) -> common::Maybe_Error {
        Ok(())
    }

    fn update(&mut self, params: Self::Update_Params);
}

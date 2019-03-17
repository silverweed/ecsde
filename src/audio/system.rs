use crate::core;
use sfml::audio as sfaud;

pub struct Audio_System {}

impl core::system::System for Audio_System {
    type Config = ();
    type Update_Params = ();

    fn update(&mut self, _: Self::Update_Params) {}
}

impl Audio_System {
    pub fn new() -> Audio_System {
        Audio_System {}
    }
}

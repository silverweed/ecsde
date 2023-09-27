use super::Phase_Args;
use inle_gfx::sprites;
use inle_app::phases::{Game_Phase, Phase_Transition, Phase_Id};
use inle_gfx::render_window::Render_Window_Handle;
use inle_math::rect::Rect;
use inle_math::vector::{lerp_v, Vec2f};
use inle_win::window;
use std::ops::DerefMut;
use std::time::Duration;

#[derive(Default)]
pub struct In_Game {
}

impl In_Game {
    pub const PHASE_ID: Phase_Id = Phase_Id::new("in_game");

    pub fn new() -> Self {
        Self { }
    }
}

impl Game_Phase for In_Game {
    type Args = Phase_Args;

    fn update(&mut self, args: &mut Self::Args) -> Phase_Transition {
        Phase_Transition::None
    }
}

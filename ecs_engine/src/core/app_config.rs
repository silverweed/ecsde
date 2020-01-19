use std::iter::Iterator;

#[cfg(debug_assertions)]
use std::path;

pub struct App_Config {
    pub title: String,
    pub target_win_size: (u32, u32),
    #[cfg(debug_assertions)]
    pub in_replay_file: Option<Box<path::Path>>,
}

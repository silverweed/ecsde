#![allow(non_camel_case_types)]

use std::env;

pub(crate) mod core;
pub(crate) mod gfx;

fn main() -> core::common::Maybe_Error {
    let cfg = core::app::Config::new(env::args());
    let mut app = core::app::App::new(&cfg);

    app.init()?;
    app.run()
}

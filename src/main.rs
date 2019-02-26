#![allow(non_camel_case_types)]

use std::env;

mod core;

fn main() -> core::common::Maybe_Error {
    let mut app = core::app::App::new();
    let cfg = core::app::Config::new(env::args());

    app.init(&cfg)?;
    app.run()
}

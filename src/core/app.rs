use super::common;

pub struct Config {}

impl Config {
    pub fn new(mut args: std::env::Args) -> Config {
        args.next();
        Config {}
    }
}

pub struct App {}

impl App {
    pub fn new() -> App {
        App {}
    }

    pub fn init(&mut self, _cfg: &Config) -> common::Maybe_Error {
        println!("Hello Sailor!");
        Ok(())
    }

    pub fn run(&mut self) -> common::Maybe_Error {
        Ok(())
    }
}

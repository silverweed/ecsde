use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let config = ecs::demo::Config::new(env::args());
	ecs::demo::run(&config);
	Ok(())
}
